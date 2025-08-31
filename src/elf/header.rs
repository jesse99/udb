//! The ELF header, it includes a mafic number, ABI details, and offsets to the different
//! parts of the ELF file.
use super::{Reader, Stream};
use crate::utils;
use std::error::Error;

// See https://refspecs.linuxfoundation.org/elf/gabi4+/ch4.eheader.html
#[derive(Debug, Default)]
pub struct ElfHeader {
    // Elf64_Ehdr, see https://llvm.org/doxygen/BinaryFormat_2ELF_8h_source.html
    /// Linux, NetBSD, etc
    pub osabi: u8,

    /// More ABI info (ignored on Linux for static exececutables).
    pub abiversion: u8, // TODO remove this?

    /// Instruction set, e.g. x86_64, ARM, etc.
    pub machine: u16,

    /// Pointer to the start of the program header table.
    pub ph_offset: u64,

    /// Size of program header table entries.
    pub ph_entry_size: u16,

    /// Number of entries in the program header table.
    pub num_ph_entries: u16,

    /// Pointer to the start of the section header table.
    pub section_offset: u64,

    /// Size of section header table entries.
    pub section_entry_size: u16,

    /// Number of entries in the section header table.
    pub num_section_entries: u16,

    /// Section index containing the string table.
    pub string_table_index: u16,

    /// Interpretation depends on the target architecture.
    pub flags: u32,
}

impl ElfHeader {
    pub fn new(reader: &Reader) -> Result<Self, Box<dyn Error>> {
        const EI_NIDENT: usize = 16;

        let mut s = Stream::new(reader, EI_NIDENT);
        let _e_type = s.read_half()?;
        let machine = s.read_half()?;
        let e_version = s.read_word()?;
        let _e_entry = s.read_addr()?;
        let ph_offset = s.read_offset()?;
        let section_offset = s.read_offset()?;
        let flags = s.read_word()?;
        let e_ehsize = s.read_half()?;
        let ph_entry_size = s.read_half()?;
        let num_ph_entries = s.read_half()?;
        let section_entry_size = s.read_half()?;
        let num_section_entries = s.read_half()?;
        let string_table_index = s.read_half()?;

        // Note that we don't expose all of the fields in the ELF header because
        // some are useless (e.g. e_shentsize) or fixed for core files (e.g. magic,
        // and e_type).
        utils::require(
            e_version == 1,
            &format!("expected e_version to be 1 not {e_version}"),
        )?;
        utils::require(
            e_ehsize == 52 || e_ehsize == 64,
            &format!("expected e_ehsize to be 52 or 64 not {e_ehsize}"),
        )?;

        Ok(ElfHeader {
            osabi: reader.read_byte(0x07)?,
            abiversion: reader.read_byte(0x08)?,
            machine,
            ph_offset,
            flags,
            ph_entry_size,
            num_ph_entries,
            section_offset,
            section_entry_size,
            num_section_entries,
            string_table_index,
        })
    }

    pub fn machine(&self) -> &'static str {
        match self.machine {
            0x00 => "No specific instruction set",
            0x01 => "AT&T WE 32100",
            0x02 => "SPARC",
            0x03 => "x86",
            0x04 => "Motorola 68000 (M68k)",
            0x05 => "Motorola 88000 (M88k)",
            0x06 => "Intel MCU",
            0x07 => "Intel 80860",
            0x08 => "MIPS",
            0x09 => "IBM System/370",
            0x0A => "MIPS RS3000 Little-endian",
            0x0F => "Hewlett-Packard PA-RISC",
            0x13 => "Intel 80960",
            0x14 => "PowerPC",
            0x15 => "PowerPC (64-bit)",
            0x16 => "S390, including S390x",
            0x17 => "IBM SPU/SPC",
            0x24 => "NEC V800",
            0x25 => "Fujitsu FR20",
            0x26 => "TRW RH-32",
            0x27 => "Motorola RCE",
            0x28 => "Arm (up to Armv7/AArch32)",
            0x29 => "Digital Alpha",
            0x2A => "SuperH",
            0x2B => "SPARC Version 9",
            0x2C => "Siemens TriCore embedded processor",
            0x2D => "Argonaut RISC Core",
            0x2E => "Hitachi H8/300",
            0x2F => "Hitachi H8/300H",
            0x30 => "Hitachi H8S",
            0x31 => "Hitachi H8/500",
            0x32 => "IA-64",
            0x33 => "Stanford MIPS-X",
            0x34 => "Motorola ColdFire",
            0x35 => "Motorola M68HC12",
            0x36 => "Fujitsu MMA Multimedia Accelerator",
            0x37 => "Siemens PCP",
            0x38 => "Sony nCPU embedded RISC processor",
            0x39 => "Denso NDR1 microprocessor",
            0x3A => "Motorola Star*Core processor",
            0x3B => "Toyota ME16 processor",
            0x3C => "STMicroelectronics ST100 processor",
            0x3D => "Advanced Logic Corp. TinyJ embedded processor family",
            0x3E => "AMD x86-64",
            0x3F => "Sony DSP Processor",
            0x40 => "Digital Equipment Corp. PDP-10",
            0x41 => "Digital Equipment Corp. PDP-11",
            0x42 => "Siemens FX66 microcontroller",
            0x43 => "STMicroelectronics ST9+ 8/16-bit microcontroller",
            0x44 => "STMicroelectronics ST7 8-bit microcontroller",
            0x45 => "Motorola MC68HC16 Microcontroller",
            0x46 => "Motorola MC68HC11 Microcontroller",
            0x47 => "Motorola MC68HC08 Microcontroller",
            0x48 => "Motorola MC68HC05 Microcontroller",
            0x49 => "Silicon Graphics SVx",
            0x4A => "STMicroelectronics ST19 8-bit microcontroller",
            0x4B => "Digital VAX",
            0x4C => "Axis Communications 32-bit embedded processor",
            0x4D => "Infineon Technologies 32-bit embedded processor",
            0x4E => "Element 14 64-bit DSP Processor",
            0x4F => "LSI Logic 16-bit DSP Processor",
            0x8C => "TMS320C6000 Family",
            0xAF => "MCST Elbrus e2k",
            0xB7 => "Arm 64-bits (Armv8/AArch64)",
            0xDC => "Zilog Z80",
            0xF3 => "RISC-V",
            0xF7 => "Berkeley Packet Filter",
            0x101 => "WDC 65C816",
            0x102 => "LoongArch",
            _ => "Unknown Machine",
        }
    }

    pub fn abi(&self) -> &'static str {
        match self.osabi {
            0x00 => "Linux", // technically "no extensions", but often used on Linux
            0x01 => "HP-UX",
            0x02 => "NetBSD",
            0x03 => "Linux",
            0x04 => "GNU Hurd", // note that there is no 0x05
            0x06 => "Solaris",
            0x07 => "AIX (Monterey)",
            0x08 => "IRIX",
            0x09 => "FreeBSD",
            0x0A => "Tru64",
            0x0B => "Novell Modesto",
            0x0C => "OpenBSD",
            0x0D => "OpenVMS",
            0x0E => "NonStop Kernel",
            0x0F => "AROS",
            0x10 => "FenixOS",
            0x11 => "Nuxi CloudABI",
            0x12 => "Stratus Technologies OpenVOS",
            _ => "unknown ABI",
        }
    }
}
