//! Used by the run-time loader. Also see sections.
use super::{Reader, Stream};
use crate::{
    elf::{Bytes, Offset, VirtualAddr},
    utils,
};
use std::error::Error;

const EXECUTE_FLAG: u32 = 0x1;
const WRITE_FLAG: u32 = 0x2;
const READ_FLAG: u32 = 0x4;

/// Describes a segment. Usually LoadSegment or Note will be used instead of this.
pub struct ProgramHeader {
    // TODO this is a rather terrible name, should we change it to SegmentHeader?
    // Elf64_Phdr or Elf32_Phdr, see https://llvm.org/doxygen/BinaryFormat_2ELF_8h_source.html
    pub stype: SegmentType,

    /// Offset to the first byte of the segment.
    pub offset: u64,

    /// Virtual address of the first byte in the segment.
    pub vaddr: u64,

    /// Physical address of the first byte in the segment.
    pub paddr: u64,

    /// Number of bytes in the segment in the core file.
    pub file_size: u64,

    /// Number of bytes in the segment in memory.
    pub mem_size: u64,

    /// Read/Write/Execute flags.
    pub flags: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SegmentType {
    /// Not to be used: either it's a segment that is intended to be not used or one
    /// that is not recognized.
    Null,

    /// A loadable segment, described by p_filesz and p_memsz.
    Load,

    /// Specifies dynamic linking information.
    Dynamic,

    /// Location and size of a null-terminated path name to invoke as an interpreter.
    Interpreter,

    /// The location and size of auxiliary information.
    Note,

    /// Reserved but has unspecified semantics.
    Shlib,

    /// The location and size of the program header table itself.
    Phdr,

    // The Thread-Local Storage template.
    Tls,
}

impl SegmentType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => SegmentType::Null,
            1 => SegmentType::Load,
            2 => SegmentType::Dynamic,
            3 => SegmentType::Interpreter,
            4 => SegmentType::Note,
            5 => SegmentType::Shlib,
            6 => SegmentType::Phdr,
            7 => SegmentType::Tls,
            0x60000000..0x70000000 => SegmentType::Null, // reserved for OS-specific semantics
            0x70000000..0x80000000 => SegmentType::Null, // reserved for processor-specific semantics
            0x80000000.. => SegmentType::Null,           // reserved for future use
            _ => {
                utils::warn(&format!("Unknown segment type: {}", value));
                SegmentType::Null
            }
        }
    }
}

pub struct LoadSegment {
    /// Addressing for the bytes in the segment using offsets from the start of the ELF file.
    pub obytes: Bytes<Offset>,

    /// Addressing for the bytes in the segment using virtual addresses as in the cored process.
    pub vbytes: Bytes<VirtualAddr>,

    // /// The physical address the segment starts at. Will be zero for core files.
    // pub paddr: u64,
    /// Readable, writeable, and/or executable.
    pub flags: u32,
}

impl LoadSegment {
    pub fn to_offset(&self, vaddr: VirtualAddr) -> Option<Offset> {
        if self.vbytes.contains(vaddr) {
            let delta = vaddr.0 - self.vbytes.start.0;
            Some(Offset(self.obytes.start.0 + delta))
        } else {
            None
        }
    }

    pub fn executable(&self) -> bool {
        self.flags & EXECUTE_FLAG != 0
    }

    pub fn writeable(&self) -> bool {
        self.flags & WRITE_FLAG != 0
    }

    pub fn readable(&self) -> bool {
        self.flags & READ_FLAG != 0
    }

    pub fn flags(&self) -> String {
        ProgramHeader::flags(self.flags)
    }
}

impl ProgramHeader {
    pub fn new(reader: &Reader, offset: Offset) -> Result<Self, Box<dyn Error>> {
        // Field sizes and order differ between 32-bit and 64-bit ELF files,
        // see https://llvm.org/doxygen/BinaryFormat_2ELF_8h_source.html.
        let mut s = Stream::new(reader, offset);
        if reader.sixty_four_bit {
            let p_type = SegmentType::from_u32(s.read_word()?);
            let p_flags = s.read_word()?;
            let p_offset = s.read_offset()?;
            let p_vaddr = s.read_addr()?;
            let p_paddr = s.read_addr()?;
            let p_filesz = s.read_xword()?;
            let p_memsz = s.read_xword()?;
            let _p_align = s.read_xword()?;
            Ok(ProgramHeader {
                stype: p_type,
                flags: p_flags,
                offset: p_offset,
                vaddr: p_vaddr,
                paddr: p_paddr,
                file_size: p_filesz,
                mem_size: p_memsz,
            })
        } else {
            let p_type = SegmentType::from_u32(s.read_word()?);
            let p_offset = s.read_offset()?;
            let p_vaddr = s.read_addr()?;
            let p_paddr = s.read_addr()?;
            let p_filesz = s.read_word()? as u64;
            let p_memsz = s.read_word()? as u64;
            let p_flags = s.read_word()?;
            let _p_align = s.read_word()? as u64;
            Ok(ProgramHeader {
                stype: p_type,
                flags: p_flags,
                offset: p_offset,
                vaddr: p_vaddr,
                paddr: p_paddr,
                file_size: p_filesz,
                mem_size: p_memsz,
            })
        }
    }

    pub fn flags(flags: u32) -> String {
        let mut result = String::new();
        if flags & EXECUTE_FLAG != 0 {
            result.push('x');
        } else {
            result.push('-');
        }
        if flags & WRITE_FLAG != 0 {
            result.push('w');
        } else {
            result.push('-');
        }
        if flags & READ_FLAG != 0 {
            result.push('r');
        } else {
            result.push('-');
        }
        result
    }
}
