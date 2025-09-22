//! Data within a core file or exe.
use super::{
    ElfHeader, LoadSegment, MemoryMappedFile, NoteType, PrStatus, ProgramHeader, Reader,
    SectionIndex, SegmentType, Stream,
};
use crate::debug::{LineInfo, SymbolTable, SymbolTableEntry};
use crate::elf::{
    Bytes, ChildSignal, CoreNoteType, FaultSignal, KillSignal, Note, Offset, PosixSignal,
    RelativeAddr, Relocation, SectionHeader, SectionType, SigInfo, SignalDetails, StringIndex,
    VirtualAddr,
};
use crate::utils::{self, warn};
use memmap2::Mmap;
use std::cell::OnceCell;
use std::error::Error;
use std::fs::File;

pub struct ElfFile {
    pub header: ElfHeader,
    pub path: std::path::PathBuf,
    pub reader: &'static Reader,
    pub loads: Vec<LoadSegment>,
    pub notes: Vec<Note>,
    pub sections: Vec<SectionHeader>, // not used for core files

    memory_mapped: OnceCell<Option<Vec<MemoryMappedFile>>>,
    lines: OnceCell<Option<LineInfo>>,
}

impl ElfFile {
    pub fn new(path: std::path::PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path.clone())?;

        // This is unfafe because it has undefined behavior if the underlying file is
        // modified while the memory map is in use.
        let bytes = unsafe { Mmap::map(&file) }?;

        // Readers will always persist for the entire life of udb so it's OK to leak them.
        // And because we leak them we can use stuff like StringView containing a static
        // reference to a Reader allowing us to not allocate memory for strings until we
        // actually need it.
        let reader = Box::leak(Box::new(Reader::new(bytes)?));
        let header = ElfHeader::new(reader)?;
        let loads = ElfFile::load_loads(reader, &header);
        let notes = ElfFile::load_notes(reader, &header);
        let sections = ElfFile::load_sections(reader, &header);
        ElfFile::load_others(reader, &header);
        Ok(ElfFile {
            path,
            reader,
            header,
            loads,
            notes,
            sections,
            memory_mapped: OnceCell::new(),
            lines: OnceCell::new(),
        })
    }

    pub fn find_load_segment(&self, vaddr: VirtualAddr) -> Option<&LoadSegment> {
        self.loads.iter().find(|s| s.vbytes.contains(vaddr))
    }

    pub fn offset_to_vaddr(&self, offset: Offset) -> Option<(&LoadSegment, VirtualAddr)> {
        self.loads
            .iter()
            .find(|s| s.obytes.contains(offset))
            .map(|s| (s, s.vbytes.start + (offset - s.obytes.start)))
    }

    pub fn vaddr_to_raddr(&self, addr: VirtualAddr) -> Option<RelativeAddr> {
        let files = self.get_memory_mapped_files();
        files.as_ref().and_then(|maps| {
            maps.iter()
                .find(|s| s.vbytes.contains(addr))
                .map(|s| RelativeAddr(addr.0 - s.vbytes.start.0))
        })
    }

    /// Returns a string from the section string table. Note that index can point into
    /// the middle of a string.
    pub fn find_default_string(&self, index: StringIndex) -> Option<String> {
        let section = SectionIndex(self.header.string_table_index as u32);
        self.find_string(section, index)
    }

    /// Returns a string from an arbitrary string table. Note that index can point into
    /// the middle of a string.
    pub fn find_string(&self, section: SectionIndex, index: StringIndex) -> Option<String> {
        let h = self.find_section(section)?;
        // TODO really should return an error if indexing past h.offset + h.size
        let offset = h.obytes.start + index.0 as i64;
        match Stream::new(self.reader, offset).read_string() {
            Ok(s) => Some(s),
            Err(err) => {
                utils::warn(&format!("failed to read section string {index:?}: {err}"));
                None
            }
        }
    }

    pub fn find_strings(&self, section: &SectionHeader, max: usize) -> Vec<String> {
        let mut result = Vec::new();
        if section.stype == SectionType::StringTable {
            let mut stream = Stream::new(self.reader, section.obytes.start);
            while stream.offset < section.obytes.end() {
                if let Ok(s) = stream.read_string() {
                    result.push(s);
                    if result.len() == max {
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn find_section_name(&self, section: SectionIndex) -> Option<String> {
        let h = self.find_section(section)?;
        self.find_default_string(h.name)
    }

    pub fn get_lines(&self) -> &Option<LineInfo> {
        self.lines.get_or_init(|| {
            for (i, section) in self.sections.iter().enumerate() {
                if section.stype == SectionType::ProgBits {
                    let index = SectionIndex(i as u32);
                    if let Some(name) = self.find_section_name(index)
                        && name == ".debug_line"
                    {
                        let max_offset = section.obytes.end();
                        return Some(LineInfo::new(
                            &mut Stream::new(self.reader, section.obytes.start),
                            max_offset,
                        ));
                    }
                }
            }
            None
        })
    }

    pub fn find_symbols(&self) -> Option<SymbolTable> {
        self.do_find_symbols(SectionType::SymbolTable)
    }

    pub fn find_dynamic_symbols(&self) -> Option<SymbolTable> {
        self.do_find_symbols(SectionType::DynamicSymbolTable)
    }

    pub fn find_segments(reader: &'static Reader, header: &ElfHeader) -> Vec<ProgramHeader> {
        let mut segments = Vec::new();
        let mut offset = Offset(header.ph_offset);

        for _ in 0..header.num_ph_entries {
            match ProgramHeader::new(reader, offset) {
                Ok(ph) => segments.push(ph),
                Err(err) => {
                    utils::warn(&format!(
                        "failed to read program header at {offset:?}: {err}"
                    ));
                }
            }
            offset = offset + header.ph_entry_size as i64;
        }
        segments
    }

    pub fn get_sections(&self) -> &Vec<SectionHeader> {
        &self.sections
    }

    pub fn get_memory_mapped_files(&self) -> &Option<Vec<MemoryMappedFile>> {
        fn get_files(s: &mut Stream) -> Result<Vec<MemoryMappedFile>, Box<dyn Error>> {
            // For some reason files get mapped in multiple times, e.g.
            //    7f45e7402000 7f45e7559000   1404928 /usr/lib64/libxpath.so
            //    7f45e7559000 7f45e7758000   2093056 /usr/lib64/libxpath.so
            //    7f45e7758000 7f45e77cd000    479232 /usr/lib64/libxpath.so
            //    7f45e77cd000 7f45e7a37000   2531328 /usr/lib64/libxpath.so
            // This is annoying and not useful so we we'll merge them together.
            // Note that the end of one line usually matches the start of the next.
            let count = s.read_ulong()?;
            let _page_size = s.read_ulong()?;

            let mut elements = Vec::new();
            for _ in 0..count {
                let start = s.read_ulong()?;
                let end = s.read_ulong()?;
                let offset = s.read_ulong()?;
                elements.push((start, end, offset));
            }

            let mut files: Vec<MemoryMappedFile> = Vec::new();
            for (start, end, _offset) in elements {
                if let Ok(file_name) = s.read_string() {
                    if let Some(old) = files.last_mut()
                        && start == old.vbytes.end().0
                        && file_name == old.file_name
                    {
                        old.vbytes.size = (end - old.vbytes.start.0) as usize;
                    } else {
                        files.push(MemoryMappedFile {
                            vbytes: Bytes::<VirtualAddr>::from_raw(start, (end - start) as usize),
                            // offset: offset * page_size,
                            file_name,
                        });
                    }
                } else {
                    utils::warn(&format!(
                        "Failed to read MemoryMappedFile at offset {:?}",
                        s.offset
                    ));
                }
            }

            Ok(files)
        }

        self.memory_mapped.get_or_init(|| {
            if let Some(note) = self.find_core_note(CoreNoteType::File) {
                let mut s = Stream::new(self.reader, note.contents.start);
                match get_files(&mut s) {
                    Ok(files) => Some(files),
                    Err(e) => {
                        utils::warn(&format!("Error reading memory mapped files: {}", e));
                        None
                    }
                }
            } else {
                None
            }
        })
    }

    pub fn find_core_note(&self, ntype: CoreNoteType) -> Option<&Note> {
        for note in self.notes.iter() {
            if let NoteType::Core(t) = &note.ntype
                && *t == ntype
            {
                return Some(note);
            }
        }
        None
    }

    pub fn find_prstatus(&self) -> Option<PrStatus> {
        fn get_prstatus(s: &mut Stream) -> Result<PrStatus, Box<dyn Error>> {
            // See elf_prstatus in https://docs.huihoo.com/doxygen/linux/kernel/3.7/uapi_2linux_2elfcore_8h_source.html
            let signal_num = s.read_int()?;
            let signal_code = s.read_int()?;
            let _errno = s.read_int()?;
            let _current_signal = s.read_half()?; // This is the current signal, not the one that caused the core dump.
            let _padding = s.read_half()?;
            let _pending_signals = s.read_xword()?;
            let _held_signals = s.read_xword()?;
            let pid = s.read_int()?;
            let _pppid = s.read_int()?;
            let _pgrp = s.read_int()?;
            let _prsid = s.read_int()?;

            let _utime_s = s.read_xword()?; // time spent in user code
            let _utime_u = s.read_xword()?;

            let _stime_s = s.read_xword()?; // time spent in system code
            let _stime_u = s.read_xword()?;

            let _cutime_s = s.read_xword()?;
            let _cutime_u = s.read_xword()?;

            let _cstime_s = s.read_xword()?;
            let _cstime_u = s.read_xword()?;

            // TODO good only for x86 and arm
            let mut registers = Vec::new();
            for _ in 1..27 {
                let r = s.read_xword()?;
                registers.push(r);
            }
            // TODO may need to use pr_exec_fdpic_loadmap

            Ok(PrStatus {
                signal_num,
                signal_code,
                // errno,
                pid,
                registers,
            })
        }

        if let Some(note) = self.find_core_note(CoreNoteType::PrStatus) {
            let mut s = Stream::new(self.reader, note.contents.start);
            match get_prstatus(&mut s) {
                Ok(status) => Some(status),
                Err(e) => {
                    utils::warn(&format!("Error reading prstatus: {}", e));
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn find_signal_info(&self) -> Option<SigInfo> {
        fn get_signal_info(s: &mut Stream) -> Result<SigInfo, Box<dyn Error>> {
            const SI_MASK: u32 = 0xffff0000;
            const SI_KILL: u32 = 0 << 16;
            const SI_TIMER: u32 = 1 << 16;
            const SI_POLL: u32 = 2 << 16;
            const SI_FAULT: u32 = 3 << 16;
            const SI_CHLD: u32 = 4 << 16;
            const SI_MESGQ: u32 = 6 << 16;
            const SI_SYS: u32 = 7 << 16;

            // See https://elixir.bootlin.com/linux/v4.9/source/arch/ia64/include/uapi/asm/siginfo.h#L83
            // and https://elixir.bootlin.com/linux/v4.9/source/arch/arm64/kernel/signal32.c#L144
            let _si_signo = s.read_xword()? as i32; // TODO I think these fields are all emitted as 64 bits?
            let _si_errno = s.read_xword()? as i32; // TODO need to test this better
            let _si_code = s.read_xword()? as i32; // TODO this seems completely wrong: we're getting kill for a seg fault

            let details = match (_si_code as u32) & SI_MASK {
                SI_KILL => {
                    let sender_pid = s.read_xword()? as i32;
                    let sender_uid = s.read_xword()? as i32;
                    SignalDetails::Kill(KillSignal {
                        sender_pid,
                        sender_uid,
                    })
                }
                SI_TIMER => SignalDetails::Timer, // TODO bit more we could include here
                SI_POLL => SignalDetails::Poll,   // TODO bit more we could include here
                SI_FAULT => {
                    let fault_addr = s.read_addr()?;
                    SignalDetails::Fault(FaultSignal { fault_addr })
                }
                SI_CHLD => {
                    let child_pid = s.read_xword()? as i32;
                    let child_uid = s.read_xword()? as i32;
                    let exit_code = s.read_xword()? as i32;
                    SignalDetails::Child(ChildSignal {
                        child_pid,
                        child_uid,
                        exit_code,
                    })
                }
                SI_MESGQ => SignalDetails::MesgQ, // TODO more we can add here
                SI_SYS => SignalDetails::Sys,     // TODO more we can add here
                _ => {
                    let sender_pid = s.read_xword()? as i32;
                    let sender_uid = s.read_xword()? as i32;
                    SignalDetails::Posix(PosixSignal {
                        sender_pid,
                        sender_uid,
                    })
                }
            };

            Ok(SigInfo {
                // signal_num: si_signo,
                // errno: si_errno,
                // signal_code: si_code,
                details,
            })
        }

        if let Some(note) = self.find_core_note(CoreNoteType::SigInfo) {
            let mut s = Stream::new(self.reader, note.contents.start);
            match get_signal_info(&mut s) {
                Ok(status) => Some(status),
                Err(e) => {
                    utils::warn(&format!("Error reading signal info: {}", e));
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn find_relocations(&self, result: &mut Vec<Relocation>) {
        fn load_with(reader: &'static Reader, offset: Offset, dynamic: bool) -> Option<Relocation> {
            match Relocation::with_addend(reader, offset, dynamic) {
                Ok(r) => Some(r),
                Err(err) => {
                    utils::warn(&format!("couldn't read relocation at {offset:?}: {err}"));
                    None
                }
            }
        }

        fn load_without(
            reader: &'static Reader,
            offset: Offset,
            dynamic: bool,
        ) -> Option<Relocation> {
            match Relocation::with_no_addend(reader, offset, dynamic) {
                Ok(r) => Some(r),
                Err(err) => {
                    utils::warn(&format!("couldn't read relocation at {offset:?}: {err}"));
                    None
                }
            }
        }

        fn load_relocations_with(
            reader: &'static Reader,
            section: &SectionHeader,
            result: &mut Vec<Relocation>,
        ) {
            let mut offset = section.obytes.start;
            while offset + section.entry_size as i64 <= section.obytes.end() {
                let dynamic = section.info == 0; // TODO better to look at section name?
                if let Some(r) = load_with(reader, offset, dynamic) {
                    result.push(r)
                }
                offset = offset + section.entry_size as i64;
            }
        }

        fn load_relocations_without(
            reader: &'static Reader,
            section: &SectionHeader,
            result: &mut Vec<Relocation>,
        ) {
            let mut offset = section.obytes.start;
            while offset + section.entry_size as i64 <= section.obytes.end() {
                let dynamic = section.info == 0; // TODO better to look at section name?
                if let Some(r) = load_without(reader, offset, dynamic) {
                    result.push(r)
                }
                offset = offset + section.entry_size as i64;
            }
        }

        if !self.header.is_x66_64() {
            utils::warn("relocations are only supported for x86 64-bit");
            return;
        }
        for section in self.sections.iter() {
            match section.stype {
                SectionType::RelocationsWith => load_relocations_with(self.reader, section, result),
                SectionType::RelocationsWithout => {
                    load_relocations_without(self.reader, section, result)
                }
                _ => (),
            }
        }
    }
}

impl ElfFile {
    fn find_section(&self, section: SectionIndex) -> Option<&SectionHeader> {
        let section_index = section.0 as usize;
        let section = self.sections.get(section_index);
        if section.is_none() {
            utils::warn(&format!("bad section index: {section_index}"));
        }
        section
    }

    fn do_find_symbols(&self, stype: SectionType) -> Option<SymbolTable> {
        for section in self.sections.iter() {
            if section.stype == stype {
                // TODO warn if there is more than one of these
                let mut offset = section.obytes.start;
                let mut entries = Vec::new();
                while offset < section.obytes.end() {
                    match SymbolTableEntry::new(self.reader, offset) {
                        Ok(s) => entries.push(s),
                        Err(err) => warn(&format!(
                            "failed to read symbols at offset {offset:?}: {err}"
                        )),
                    }
                    offset = offset + section.entry_size as i64;
                }
                let table = SymbolTable {
                    section: section.clone(),
                    dynamic: stype == SectionType::DynamicSymbolTable,
                    entries,
                };
                return Some(table);
            }
        }
        None
    }

    fn load_loads(reader: &'static Reader, header: &ElfHeader) -> Vec<LoadSegment> {
        let mut loads = Vec::new();
        let mut offset = Offset(header.ph_offset);

        // Even a large core file has a small number of program headers, so it's OK to
        // re-iterate over them.
        for _ in 0..header.num_ph_entries {
            match ProgramHeader::new(reader, offset) {
                Ok(ph) => {
                    if ph.stype == SegmentType::Load {
                        let obytes = Bytes::<Offset>::from_raw(ph.offset, ph.mem_size as usize);
                        let vbytes = Bytes::<VirtualAddr>::from_raw(ph.vaddr, ph.mem_size as usize);
                        loads.push(LoadSegment {
                            obytes,
                            vbytes,
                            // paddr: ph.paddr,
                            flags: ph.flags,
                        });
                    }
                }
                Err(err) => utils::warn(&format!(
                    "failed to read program header at {offset:?}: {err}"
                )),
            }
            offset = offset + header.ph_entry_size as i64;
        }
        loads
    }

    fn load_notes(reader: &'static Reader, header: &ElfHeader) -> Vec<Note> {
        fn load_note(s: &mut Stream) -> Option<Note> {
            if let Ok((name, ntype, contents)) = super::read_note(s) {
                Some(Note {
                    name: name.clone(),
                    ntype: NoteType::new(&name, ntype),
                    contents,
                })
            } else {
                utils::warn(&format!("Failed to read note at offset {:?}", s.offset));
                None
            }
        }

        let mut notes = Vec::new();
        let mut offset = Offset(header.ph_offset);

        for _ in 0..header.num_ph_entries {
            match ProgramHeader::new(reader, offset) {
                Ok(ph) => {
                    // Note that core files can sometimes be damaged (typically because they are
                    // truncated). Not all notes are essential so we'll try to continue even if
                    // a note cannot be read.
                    if ph.stype == SegmentType::Note {
                        let start = Offset(ph.offset);
                        let end = start + ph.file_size as i64;
                        let mut s = Stream::new(reader, start);
                        while s.offset < end {
                            if let Some(note) = load_note(&mut s) {
                                // TODO may want to warn if get a second note of the same type
                                notes.push(note);
                            }
                        }
                    }
                }
                Err(err) => utils::warn(&format!(
                    "failed to read program header at {offset:?}: {err}"
                )),
            }
            offset = offset + header.ph_entry_size as i64;
        }
        notes
    }

    // This is just here so we can report unknown segments.
    fn load_others(reader: &'static Reader, header: &ElfHeader) {
        let mut offset = Offset(header.ph_offset);

        for _ in 0..header.num_ph_entries {
            match ProgramHeader::new(reader, offset) {
                Ok(ph) => match ph.stype {
                    SegmentType::Dynamic => (), // TODO may need to use this one
                    SegmentType::Interpreter => (),
                    SegmentType::Note => (),
                    SegmentType::Null => (),
                    SegmentType::Load => (),
                    SegmentType::Phdr => (),
                    _ => utils::warn(&format!("Ignoring segment type {:?}", ph.stype)),
                },
                Err(err) => utils::warn(&format!(
                    "failed to read program header at {offset:?}: {err}"
                )),
            }
            offset = offset + header.ph_entry_size as i64;
        }
    }

    fn load_sections(reader: &'static Reader, header: &ElfHeader) -> Vec<SectionHeader> {
        let mut sections = Vec::new();
        let mut offset = Offset(header.section_offset);

        for _ in 0..header.num_section_entries {
            match SectionHeader::new(reader, offset) {
                Ok(h) => sections.push(h),

                Err(err) => utils::warn(&format!(
                    "failed to read section header at {offset:?}: {err}"
                )),
            }
            offset = offset + header.section_entry_size as i64;
        }
        sections
    }
}

// TODO may want to add an option to Reader so that it (randomly?) fails to see how we
// handle corrupt files, or maybe better to simulate truncation
#[cfg(test)]
mod tests {
    use super::*;

    // TODO duplicate these for the release core? or is there something we can do to
    // parameterize it with insta? or maybe just have the tests do both cores?
    #[test]
    fn debug_header() {
        let path = std::path::PathBuf::from("cores/shopping-debug/app-debug.core");
        let core = ElfFile::new(path).unwrap();
        let s = format!("{} on {}", core.header.machine(), core.header.abi());
        insta::assert_snapshot!(s);
    }

    #[test]
    fn debug_signal() {
        let path = std::path::PathBuf::from("cores/shopping-debug/app-debug.core");
        let core = ElfFile::new(path).unwrap();
        let s = match core.find_prstatus() {
            Some(status) => status.signal(),
            None => "no pr status",
        };
        insta::assert_snapshot!(s);
    }

    #[test]
    fn debug_memory_mapped_files() {
        let path = std::path::PathBuf::from("cores/shopping-debug/app-debug.core");
        let core = ElfFile::new(path).unwrap();
        let s = match core.get_memory_mapped_files() {
            // start_addr and end_addr will change with each build
            // size will change if we tweak the code
            // so we won't test any of that
            Some(files) => files.iter().map(|f| format!("{}\n", f.file_name)).collect(),
            None => "no files".to_string(),
        };
        insta::assert_snapshot!(s);
    }
}
