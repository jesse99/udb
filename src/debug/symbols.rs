use crate::{
    elf::{Offset, Reader, SectionHeader, SectionIndex, Stream, StringIndex},
    utils,
};
use std::error::Error;

pub struct SymbolTable {
    pub section: SectionHeader,
    pub dynamic: bool,
    pub entries: Vec<SymbolTableEntry>,
}

pub struct SymbolTableEntry {
    // see https://refspecs.linuxbase.org/elf/gabi4+/ch4.symtab.html
    /// Index into the symbol string table.
    pub name: StringIndex,

    /// Can be an address, absolute value, etc.
    pub value: u64,

    /// Size of the symbol. Zero if the symbol has no or unknown size.
    pub size: u64,

    pub stype: SymbolType,

    pub binding: SymbolBinding,

    pub visibility: SymbolVisibility,

    pub index: SymbolIndex,
}

#[derive(Clone, Copy, Debug)]
pub enum SymbolIndex {
    /// Symbol has an absolute value that will not change with relocation.
    Abs,

    /// A common block that has not yet been allocated. Value has alignment.
    Common,

    /// Symbol value refers to another section at this index.
    Index(SectionIndex),

    /// Value is undefined. Linker will fix these up.
    Undef,

    /// Used when Index overflows. Related section will be of type SHT_SYMTAB_SHNDX.
    XIndex,
}

#[derive(Debug)]
pub enum SymbolVisibility {
    /// Visibility is per binding.
    Default,

    /// Visible only within its object file. CPU may special case this.
    Internal,

    /// Visible only within its object file.
    Hidden,

    /// Visible to other object files but cannot be prempted.
    Protected,
}

/// Linkage visibility and behavior
#[derive(Debug)]
pub enum SymbolBinding {
    /// Symbol is not visible outside the object file containing its definition. These
    /// will appear before global and weak symbols in the table.
    Local,

    /// Visible to all object files.
    Global,

    /// Similar to Global but has lower precedence. These can be preempted by a Global.
    Weak,

    /// For use by OS or CPU.
    Reserved,
}

#[derive(Debug)]
pub enum SymbolType {
    None,

    /// A data object, variable, array, etc.
    Object,

    /// Function or other executable code.
    Func,

    /// Another section. Used for relocation.
    Section,

    /// Source file associated with the symbol table.
    File,

    /// Uninitialized common blocks. Used by the linker.
    Common,

    /// Thread Local Storage data. Value is an offset to the data.
    Tls,

    /// For use by OS or CPU.
    Reserved,
}

impl SymbolTableEntry {
    pub fn new(reader: &'static Reader, offset: Offset) -> Result<Self, Box<dyn Error>> {
        // Field order is different so we need both cases.
        let mut s = Stream::new(reader, offset);
        if reader.sixty_four_bit {
            let name = s.read_word()?; // 4
            let info = s.read_byte()?; // 1
            let other = s.read_byte()?; // 1
            let index = s.read_half()?; // 2
            let value = s.read_addr()?; // 8
            let size = s.read_xword()?; // 8
            Ok(SymbolTableEntry {
                name: StringIndex(name),
                value,
                size,
                stype: SymbolType::from_u8(info),
                binding: SymbolBinding::from_u8(info),
                visibility: SymbolVisibility::from_u8(other),
                index: SymbolIndex::from_u16(index),
            })
        } else {
            let name = s.read_word()?;
            let value = s.read_addr()?;
            let size = s.read_word()? as u64;
            let info = s.read_byte()?;
            let other = s.read_byte()?;
            let index = s.read_half()?;
            Ok(SymbolTableEntry {
                name: StringIndex(name),
                value,
                size,
                stype: SymbolType::from_u8(info),
                binding: SymbolBinding::from_u8(info),
                visibility: SymbolVisibility::from_u8(other),
                index: SymbolIndex::from_u16(index),
            })
        }
    }
}

impl SymbolIndex {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0 => SymbolIndex::Undef,
            0xfff1 => SymbolIndex::Abs,
            0xfff2 => SymbolIndex::Common,
            0xffff => SymbolIndex::XIndex,
            _ => SymbolIndex::Index(SectionIndex(value as u32)),
        }
    }
}

impl SymbolVisibility {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SymbolVisibility::Default,
            1 => SymbolVisibility::Internal,
            2 => SymbolVisibility::Hidden,
            3 => SymbolVisibility::Protected,
            _ => {
                utils::warn(&format!("Unknown symbol visibility: {}", value));
                SymbolVisibility::Default
            }
        }
    }
}

impl SymbolBinding {
    pub fn from_u8(value: u8) -> Self {
        match value >> 4 {
            0 => SymbolBinding::Local,
            1 => SymbolBinding::Global,
            2 => SymbolBinding::Weak,
            10 | 12 | 13 | 15 => SymbolBinding::Reserved,
            _ => {
                utils::warn(&format!("Unknown symbol binding: {}", value >> 4));
                SymbolBinding::Reserved
            }
        }
    }
}

impl SymbolType {
    pub fn from_u8(value: u8) -> Self {
        match value & 0xf {
            0 => SymbolType::None,
            1 => SymbolType::Object,
            2 => SymbolType::Func,
            3 => SymbolType::Section,
            4 => SymbolType::File,
            5 => SymbolType::Common,
            6 => SymbolType::Tls,
            10 | 12 | 13 | 15 => SymbolType::Reserved,
            _ => {
                utils::warn(&format!("Unknown symbol type: {}", value & 0xf));
                SymbolType::Reserved
            }
        }
    }
}
