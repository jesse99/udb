//! This module contains support for the debugging support encoded into ELF files. Most
//! of this is in exe files, not core files. Most of this info is encoded into ".debug_FOO"
//! sections, e.g. ".debug_info", ".debug_abbrev", etc. These contain dwarf debug info
//! which are documented here: https://dwarfstd.org/doc/DWARF5.pdf. The readelf source
//! code is also useful and can be found at https://github.com/bminor/binutils-gdb/tree/master/binutils.
pub mod line;
pub mod symbols;

pub use line::*;
pub use symbols::*;
