//! Used by the linker and debugger. Also see segments.
use super::{Reader, Stream};
use crate::{
    elf::{Bytes, ElfOffset, VirtualAddr},
    utils,
};
use std::error::Error;

const WRITE_FLAG: u64 = 1 << 0; // Writable
const ALLOC_FLAG: u64 = 1 << 1; // Occupies memory during execution
const EXECINSTR_FLAG: u64 = 1 << 2; // Executable
const MERGE_FLAG: u64 = 1 << 4; // Might be merged
const STRINGS_FLAG: u64 = 1 << 5; // Contains nul-terminated strings
const INFO_LINK_FLAG: u64 = 1 << 6; // `sh_info' contains SHT index
const LINK_ORDER_FLAG: u64 = 1 << 7; // Preserve order after combining
const OS_NONCONFORMING_FLAG: u64 = 1 << 8; // Non-standard OS specific handling required
const GROUP_FLAG: u64 = 1 << 9; // Section is member of a group. 
const TLS_FLAG: u64 = 1 << 10; // Section hold thread-local data. 
const COMPRESSED_FLAG: u64 = 1 << 11; // Section with compressed data.
const MASKOS_FLAG: u64 = 0x0ff00000; // OS-specific. 
const MASKPROC_FLAG: u64 = 0xf0000000; // Processor-specific

/// Describes a section.
#[derive(Clone)]
pub struct SectionHeader {
    // Elf32_Shdr or Elf64_Shdr, see hthttps://gist.github.com/x0nu11byt3/bcb35c3de461e5fb66173071a2379779
    /// Index into the string table. Zero means no name.
    pub name: u32,

    /// Type of the section.
    pub stype: SectionType,

    /// Write, alloc, and/or exec.
    pub flags: u64,

    /// Addressing for the bytes in the segment using offsets from the start of the ELF file.
    pub obytes: Bytes<ElfOffset>,

    /// Addressing for the bytes in the segment using virtual addresses as in the cored process.
    pub vbytes: Bytes<VirtualAddr>,

    /// Link to another section with related information, usually a string
    /// or symbol table.
    pub link: u32,

    /// Additional section info.
    pub info: u32,

    /// Section alignment.
    pub align: u64,

    /// Set if the section holds a table of entries.
    pub entry_size: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionType {
    /// Dynamic linking information.
    Dynamic,

    // Dynamic linker symbol table.
    DynamicSymbolTable,

    /// Array of pointers to termination functions.
    FiniArray,

    /// GNU style hash table.
    Hash,

    /// Array of pointers to initialization functions.
    InitArray,

    /// Uninitialized data.
    NoBits,

    /// Arbitrary metadata.
    Note,

    /// Not to be used.
    Null,

    /// Array of pointers to functions to be called before the regular
    /// initialization functions.
    PreinitArray,

    /// CPU instructions or constant data.
    ProgBits,

    /// Relocation entries with addends.
    RelocationsWith,

    /// Relocation entries without addends.
    RelocationsWithout,

    /// Strings for use by the linker and debugger.
    StringTable,

    /// Symbol hash table.
    SymbolHashTable,

    /// Debugging info.
    SymbolTable,

    /// GNU symbol versions that are provided.
    VerDef,

    /// GNU symbol versions that are required.
    VerNeed,

    /// GNU symbol version table.
    VerSym,
}

impl SectionType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x6 => SectionType::Dynamic, // see https://android.googlesource.com/platform/art/+/e34fa1d/runtime/elf.h
            0xb => SectionType::DynamicSymbolTable,
            0xf => SectionType::FiniArray,
            0x5 => SectionType::SymbolHashTable,
            0xe => SectionType::InitArray,
            0x8 => SectionType::NoBits,
            0x7 => SectionType::Note,
            0x0 => SectionType::Null,
            0x10 => SectionType::PreinitArray,
            0x1 => SectionType::ProgBits,
            0x9 => SectionType::RelocationsWithout,
            0x4 => SectionType::RelocationsWith,
            0x3 => SectionType::StringTable,
            0x2 => SectionType::SymbolTable,
            0x6ffffff6 => SectionType::Hash,
            0x6ffffffd => SectionType::VerDef,
            0x6ffffffe => SectionType::VerNeed,
            0x6fffffff => SectionType::VerSym,
            _ => {
                utils::warn(&format!("Unknown section type: {}", value));
                SectionType::Null
            }
        }
    }
}

impl SectionHeader {
    pub fn flags(flags: u64) -> String {
        let mut result = Vec::new();
        if flags & WRITE_FLAG != 0 {
            result.push("WRITE");
        }
        if flags & ALLOC_FLAG != 0 {
            result.push("ALLOC");
        }
        if flags & EXECINSTR_FLAG != 0 {
            result.push("EXEC");
        }
        if flags & MERGE_FLAG != 0 {
            result.push("MERGE");
        }
        if flags & STRINGS_FLAG != 0 {
            result.push("STRINGS");
        }
        if flags & INFO_LINK_FLAG != 0 {
            result.push("INFO");
        }
        if flags & LINK_ORDER_FLAG != 0 {
            result.push("LINK");
        }
        if flags & OS_NONCONFORMING_FLAG != 0 {
            result.push("OS_NONCONFORMING");
        }
        if flags & GROUP_FLAG != 0 {
            result.push("GROUP");
        }
        if flags & TLS_FLAG != 0 {
            result.push("TLS");
        }
        if flags & COMPRESSED_FLAG != 0 {
            result.push("COMPRESSED");
        }
        if flags & MASKOS_FLAG != 0 {
            result.push("MASKOS");
        }
        if flags & MASKPROC_FLAG != 0 {
            result.push("MASKPROC");
        }
        if result.is_empty() {
            result.push("none");
        }
        result.join(" ")
    }
}

impl SectionHeader {
    pub fn new(reader: &Reader, offset: usize) -> Result<Self, Box<dyn Error>> {
        let mut s = Stream::new(reader, offset);
        if reader.sixty_four_bit {
            let name = s.read_word()?;
            let stype = SectionType::from_u32(s.read_word()?);
            let flags = s.read_xword()?;
            let vaddr = s.read_addr()?;
            let offset = s.read_offset()?;
            let size = s.read_xword()?;
            let link = s.read_word()?;
            let info = s.read_word()?;
            let align = s.read_xword()?;
            let entry_size = s.read_xword()?;
            Ok(SectionHeader {
                name,
                stype,
                flags,
                obytes: Bytes::<ElfOffset>::from_raw(offset, size as usize),
                vbytes: Bytes::<VirtualAddr>::from_raw(vaddr, size as usize),
                link,
                info,
                align,
                entry_size,
            })
        } else {
            let name = s.read_word()?;
            let stype = SectionType::from_u32(s.read_word()?);
            let flags = s.read_word()? as u64;
            let vaddr = s.read_addr()?;
            let offset = s.read_offset()?;
            let size = s.read_word()? as u64;
            let link = s.read_word()?;
            let info = s.read_word()?;
            let align = s.read_word()? as u64;
            let entry_size = s.read_word()? as u64;
            Ok(SectionHeader {
                name,
                stype,
                flags,
                obytes: Bytes::<ElfOffset>::from_raw(offset, size as usize),
                vbytes: Bytes::<VirtualAddr>::from_raw(vaddr, size as usize),
                link,
                info,
                align,
                entry_size,
            })
        }
    }
}

// see https://intezer.com/blog/executable-and-linkable-format-101-part-3-relocations/
#[derive(Debug)]
pub struct Relocation {
    pub offset: u64,
    pub dynamic: bool,
    pub symbol_index: u32,
    pub rtype: RelocationX86_64,
    pub addend: Option<i64>,
}

#[derive(Debug)]
pub enum RelocationX86_64 {
    // name        val  field   calculation
    None,       // 0	None	None
    SixtyFour,  // 1	qword	S + A
    Pc32,       // 2	dword	S + A – P
    Got32,      // 3	dword	G + A
    Plt32,      // 4	dword	L + A – P
    Copy,       // 5	None	Value is copied directly from shared object
    GlobDat,    // 6	qword	S
    JumpSlot,   // 7	qword	S
    Relative,   // 8	qword	B + A
    GotPcRel,   // 9	dword	G + GOT + A – P
    ThirtyTwo,  // 10	dword	S + A
    ThirtyTwoS, // 11	dword	S + A
    Sixteen,    // 12	word	S + A
    Pc16,       // 13	word	S + A – P
    Eight,      // 14	word8	S + A
    Pc8,        // 15	word8	S + A – P
    Pc64,       // 24	qword	S + A – P
    GoTOoff64,  // 25	qword	S + A – GOT
    GotPc32,    // 26	dword	GOT + A – P
    Size32,     // 32	dword	Z + A
    Size64,     // 33	qword	Z + A
}

impl Relocation {
    pub fn with_no_addend(
        reader: &Reader,
        offset: usize,
        dynamic: bool,
    ) -> Result<Self, Box<dyn Error>> {
        Relocation::new(reader, offset, false, dynamic)
    }

    pub fn with_addend(
        reader: &Reader,
        offset: usize,
        dynamic: bool,
    ) -> Result<Self, Box<dyn Error>> {
        Relocation::new(reader, offset, true, dynamic)
    }

    fn new(
        reader: &Reader,
        offset: usize,
        has_addend: bool,
        dynamic: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let mut s = Stream::new(reader, offset);
        let offset = s.read_addr()?;
        let info = s.read_xword()?;
        let addend = if has_addend {
            Some(s.read_sxword()?)
        } else {
            None
        };
        if reader.sixty_four_bit {
            Ok(Relocation {
                offset,
                symbol_index: (info >> 32) as u32,
                rtype: RelocationX86_64::from_u64(info & 0xffffffff)?,
                addend,
                dynamic,
            })
        } else {
            Ok(Relocation {
                offset,
                symbol_index: (info >> 8) as u32,
                rtype: RelocationX86_64::from_u64(info & 0xff)?,
                addend,
                dynamic,
            })
        }
    }
}

impl RelocationX86_64 {
    fn from_u64(rtype: u64) -> Result<Self, Box<dyn Error>> {
        match rtype {
            0 => Ok(RelocationX86_64::None),
            1 => Ok(RelocationX86_64::SixtyFour),
            2 => Ok(RelocationX86_64::Pc32),
            3 => Ok(RelocationX86_64::Got32),
            4 => Ok(RelocationX86_64::Plt32),
            5 => Ok(RelocationX86_64::Copy),
            6 => Ok(RelocationX86_64::GlobDat),
            7 => Ok(RelocationX86_64::JumpSlot),
            8 => Ok(RelocationX86_64::Relative),
            9 => Ok(RelocationX86_64::GotPcRel),
            10 => Ok(RelocationX86_64::ThirtyTwo),
            11 => Ok(RelocationX86_64::ThirtyTwoS),
            12 => Ok(RelocationX86_64::Sixteen),
            13 => Ok(RelocationX86_64::Pc16),
            14 => Ok(RelocationX86_64::Eight),
            15 => Ok(RelocationX86_64::Pc8),
            24 => Ok(RelocationX86_64::Pc64),
            25 => Ok(RelocationX86_64::GoTOoff64),
            26 => Ok(RelocationX86_64::GotPc32),
            32 => Ok(RelocationX86_64::Size32),
            33 => Ok(RelocationX86_64::Size64),
            _ => Err(format!("bad x86 64 relocation type: {rtype}").into()),
        }
    }
}
