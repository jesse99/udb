//! Used by the linker and debugger. Also see segments.
use super::{Reader, Stream};
use crate::utils;
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

    /// Virtual address at execution.
    pub vaddr: u64,

    /// Offset into the ELF file for the start of the section.
    pub offset: u64,

    /// Section size in bytes.
    pub size: u64,

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
                vaddr,
                offset,
                size,
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
                vaddr,
                offset,
                size,
                link,
                info,
                align,
                entry_size,
            })
        }
    }
}
