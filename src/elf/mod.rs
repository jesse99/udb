//! Generic ELF file support. These can be both core files and executable files.
//! (The executable files are useful because they contain the bulk of the debugging
//! data needed to interpret the core files.) This module contains generic ELF suport
//! but not the support for stuff like interpreting debug symbols.
//! Quick ELF reference: https://gist.github.com/x0nu11byt3/bcb35c3de461e5fb66173071a2379779
//!
//! ELF files start with an ELF header which includes:
//! * A magic number to identify the file as an ELF file.
//! * The architecture, e.g. Linux AMD x86-64.
//! * The offset to and number of program headers.
//! * The offset to and number of section headers.
//!
//! Program headers identify segments. Segments are used by the OS to load an exe into
//! memory. A program header has type, vaddr, offset, etc. Common types are:
//! * Load - for a core file these are usually memory mapped files for the exe and DLLs, for an exe these are text (CPU instructions) and data (eg statics)
//! * Note - variety of metadata, e.g. process and signal info.
//! * TLS - thread local storage info.
//!
//! Section headers identify sections. Sections are used for static linking and don't
//! appear in core files. Section headers have name, type, vaddr, offset, size, etc.
//! There are a lot of types including for the symbol table, string table, etc.
pub mod elf_file;
pub mod elf_files;
pub mod header;
pub mod io;
pub mod notes;
pub mod sections;
pub mod segments;

pub use elf_file::*;
pub use elf_files::*;
pub use header::*;
pub use io::*;
pub use notes::*;
pub use sections::*;
pub use segments::*;
