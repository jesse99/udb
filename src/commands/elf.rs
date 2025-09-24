use super::tables::{add_field, add_simple};
use crate::commands::tables::{SimpleTableBuilder, TableBuilder};
use crate::debug::SymbolIndex;
use crate::elf::{
    LoadSegment, MemoryMappedFile, ProgramHeader, SectionHeader, SectionType, StringIndex,
    VirtualAddr,
};
use crate::repl::{DebugArgs, ExplainArgs, StringsArgs};
use crate::utils;
use crate::utils::Styling;
use crate::{elf::ElfFile, elf::ElfFiles, repl::TableArgs};

fn get_file(files: &ElfFiles, exe: bool) -> &ElfFile {
    if exe {
        match &files.exe {
            Some(file) => file,
            None => {
                utils::warn("--exe was used but there is no exe: using core instead");
                files.core.as_ref().unwrap()
            }
        }
    } else {
        match &files.core {
            Some(file) => file,
            None => files.exe.as_ref().unwrap(),
        }
    }
}

pub fn info_debug(files: &ElfFiles, args: &DebugArgs) {
    let file = get_file(files, true);
    match file.get_lines() {
        Some(lines) => {
            for (i, unit) in lines.units.iter().enumerate() {
                println!("compilation unit {i}:");
                println!("   sources:");
                for source in unit.source_files.iter() {
                    match source.length {
                        Some(n) => println!("      {}/{} {} bytes", source.dir, source.file, n),
                        None => println!("      {}/{}", source.dir, source.file),
                    }
                }
                println!("   include paths:");
                for i in unit.include_paths.iter() {
                    println!("      {}", i);
                }
            }
            println!("files:");
            for f in lines.files.iter() {
                println!("   {}", f);
            }
            println!("relative addresses:");
            for (a, v) in lines.lines.iter().take(args.max_lines) {
                let f = &lines.files.get(v.file);
                println!("   0x{:x}  {}:{}:{}", a.start.0, f, v.line, v.column);
            }
            if lines.lines.len() > args.max_lines {
                println!("   ...");
            }
        }
        None => println!("Couldn't find .debug_line section"),
    }
}

pub fn info_header(files: &ElfFiles, args: &ExplainArgs) {
    let mut b = SimpleTableBuilder::new();

    let file = get_file(files, args.exe);
    add_simple!(b, "type", file.header.stype(), "type of ELF file");
    if file.reader.little_endian {
        add_simple!(
            b,
            "little endian",
            file.reader.little_endian,
            "words are being laid out in memory with the most significant byte last"
        );
    } else {
        add_simple!(
            b,
            "little endian",
            file.reader.little_endian,
            "words are being laid out out in memory with the most significant byte first"
        );
    }
    if file.reader.sixty_four_bit {
        add_simple!(
            b,
            "64-bit",
            file.reader.sixty_four_bit,
            "pointers are eight bytes"
        );
    } else {
        add_simple!(
            b,
            "64-bit",
            file.reader.sixty_four_bit,
            "pointers are four bytes"
        );
    }
    add_simple!(
        b,
        "osabi",
        file.header.abi(),
        "the OS the binary was compiled for"
    );
    add_simple!(b, "abiversion", file.header.abiversion, "zero for Linux");
    add_simple!(b, "machine", file.header.machine(), "CPU architecture");
    add_simple!(b, "flags", file.header.flags, "Linux has no defined flags");
    add_simple!(
        b,
        "ph_offset",
        file.header.ph_offset,
        "offset in the ELF file to the Program Header table"
    );
    add_simple!(
        b,
        "num_ph_entries",
        file.header.num_ph_entries,
        "number of entries in the Program Header table"
    );
    add_simple!(
        b,
        "section_offset",
        file.header.section_offset,
        "offset in the ELF file to the section header table"
    );
    add_simple!(
        b,
        "num_section_entries",
        file.header.num_section_entries,
        "number of entries in the section header table"
    );
    add_simple!(
        b,
        "string_table_index",
        file.header.string_table_index,
        "section index containing the string table"
    );
    b.println(args.explain);
}

pub fn info_loads(files: &ElfFiles, args: &TableArgs) {
    pub fn find_file(
        files: &Option<Vec<MemoryMappedFile>>,
        vaddr: VirtualAddr,
    ) -> Option<&MemoryMappedFile> {
        if let Some(maps) = files {
            maps.iter().find(|m| m.vbytes.contains(vaddr))
        } else {
            None
        }
    }

    pub fn is_stack(file: &ElfFile, segment: &LoadSegment) -> bool {
        if let Some(status) = file.find_prstatus() {
            let bottom = status.get_frame_stack_bottom();
            segment.vbytes.contains(bottom)
        } else {
            false
        }
    }

    let mut builder = TableBuilder::new();
    builder.add_col_l("vaddr", "the virtual address the segment starts at");
    builder.add_col_r("vbytes", "the size of the segment in memory");
    builder.add_col_r("flags", "executable, writeable, and/or readable");
    builder.add_col_r(
        "offset",
        "the offset into the ELF file at which the segment appears",
    );
    builder.add_col_r("obytes", "the size of the segment in the core");
    builder.add_col_l(
        "note",
        "path to memory mapped file or how the segment is used",
    );

    let file = get_file(files, args.exe);
    let files = file.get_memory_mapped_files();
    for segment in file.loads.iter() {
        let mut note = String::new();
        if let Some(file) = find_file(files, segment.vbytes.start) {
            note.push_str(&format!("{} ", file.file_name));
        } else if is_stack(file, segment) {
            note.push_str("[stack] ");
        } else if !segment.executable() && segment.writeable() && segment.readable() {
            note.push_str("[data] ");
        } else if !segment.executable() && !segment.writeable() && segment.readable() {
            // TODO may also want to check that the first bytes are '.ELF'
            note.push_str("[text] ");
        }

        add_field!(builder, "vaddr", "{:x}", segment.vbytes.start.0);
        add_field!(builder, "vbytes", "{:x}", segment.vbytes.size);
        add_field!(builder, "flags", segment.flags());
        add_field!(builder, "offset", "{:x}", segment.obytes.start.0);
        add_field!(builder, "obytes", "{:x}", segment.obytes.size);
        add_field!(builder, "note", note);
    }

    builder.println(args.titles, args.explain);
}

pub fn info_notes(files: &ElfFiles, args: &TableArgs) {
    let mut builder = TableBuilder::new();
    builder.add_col_l("name", "note namespace");
    builder.add_col_l("type", "the type of the note");
    builder.add_col_r("offset", "offset into the ELF file (hex)");
    builder.add_col_r("size", "size of the note");

    let file = get_file(files, args.exe);
    for note in file.notes.iter() {
        add_field!(builder, "name", note.name);
        add_field!(builder, "type", "{:?}", note.ntype);
        add_field!(builder, "offset", "{:x}", note.contents.start.0);
        add_field!(builder, "size", note.contents.size);
    }

    builder.println(args.titles, args.explain);
}

pub fn info_relocations(files: &ElfFiles, args: &TableArgs) {
    // TODO probably should use an arg w/o --exe
    let mut builder = TableBuilder::new();
    builder.add_col_r("symbol", "name of the symbol to relocate");
    builder.add_col_r("index", "index of the symbol to relocate"); // TODO get rid of this
    builder.add_col_r("string", "index of the symbol string"); // TODO get rid of this
    builder.add_col_r("dynamic", "true if the symbol is from a shared library");
    builder.add_col_r("offset", "vaddr for exe or shared object");
    builder.add_col_l("type", "how to apply the relocation (arch specific)");
    builder.add_col_r("addend", "optional constant applied during relocation");

    let file = get_file(files, true);
    let symbols = file.find_symbols();
    let dynamic_symbols = file.find_dynamic_symbols();

    let rels = files.find_relocations();
    for r in rels.iter() {
        // TODO names aren't great for static relocation entries. They do match what
        // `readelf --syms` reports but they are sucky names. For example,
        // `readelf --relocs` will report "printf@GLIBC_2.2.5" but we say
        // "deregister_tm_clones".
        let name = if r.dynamic {
            match &dynamic_symbols {
                Some(t) => {
                    let e = t.entries.get(r.symbol as usize);
                    e.map(|ue| file.find_string(t.section.link, ue.name))
                }
                None => None,
            }
        } else {
            match &symbols {
                Some(t) => {
                    let e = t.entries.get(r.symbol as usize);
                    e.map(|ue| file.find_string(t.section.link, ue.name))
                }
                None => None,
            }
        }
        .flatten()
        .unwrap_or(format!("index {}", r.symbol));

        let string = if r.dynamic {
            match &dynamic_symbols {
                Some(t) => {
                    let e = t.entries.get(r.symbol as usize);
                    e.map(|ue| ue.name)
                }
                None => None,
            }
        } else {
            match &symbols {
                Some(t) => {
                    let e = t.entries.get(r.symbol as usize);
                    e.map(|ue| ue.name)
                }
                None => None,
            }
        }
        .unwrap_or(StringIndex(0));

        let addend = match r.addend {
            Some(a) => format!("{}", a),
            None => "none".to_string(),
        };
        add_field!(builder, "symbol", name);
        add_field!(builder, "dynamic", r.dynamic);
        add_field!(builder, "index", r.symbol);
        add_field!(builder, "string", string.0);
        add_field!(builder, "offset", "{:x}", r.offset);
        add_field!(builder, "type", "{:?}", r.rtype);
        add_field!(builder, "addend", addend);
    }

    builder.println(args.titles, args.explain);
}

pub fn info_sections(files: &ElfFiles, args: &TableArgs) {
    let file = get_file(files, args.exe);
    let sections = file.get_sections();

    let mut builder = TableBuilder::new();
    builder.add_col_r("index", "index into sections.");
    builder.add_col_l("name", "index into the string table. Zero means no name.");
    builder.add_col_l("type", "type of the section.");
    builder.add_col_r("vaddr", "virtual address at execution.");
    builder.add_col_r(
        "offset",
        "offset into the ELF file for the start of the section.",
    );
    builder.add_col_r("size", "section size in bytes.");
    builder.add_col_r("entry_size", "set if the section holds a table of entries.");
    builder.add_col_r("align", "section alignment.");
    builder.add_col_r(
        "link",
        "link to another section with related information, usually a string or symbol table.",
    );
    builder.add_col_r("info", "additional section info");
    builder.add_col_l("flags", "write, alloc, and/or exec.");

    // Would be kind of nice to sort these by name but they are referenced sometimes
    // by index...
    for (i, section) in sections.iter().enumerate() {
        add_field!(builder, "index", i); // sections are often referenced by index so this is handy
        match file.find_default_string(section.name) {
            Some(n) => {
                add_field!(builder, "name", n);
            }
            None => {
                add_field!(builder, "name", section.name.0);
            }
        };
        add_field!(builder, "type", "{:?}", section.stype);
        add_field!(builder, "flags", SectionHeader::flags(section.flags));
        add_field!(builder, "vaddr", "{:x}", section.vbytes.start.0);
        add_field!(builder, "offset", "{:x}", section.obytes.start.0);
        add_field!(builder, "size", section.vbytes.size);
        add_field!(builder, "entry_size", section.entry_size);
        add_field!(builder, "align", section.align);
        add_field!(builder, "link", section.link.0);
        add_field!(builder, "info", section.info);
    }

    builder.println(args.titles, args.explain);
}

pub fn info_segments(files: &ElfFiles, args: &TableArgs) {
    let file = get_file(files, args.exe);
    let segments = ElfFile::find_segments(file.reader, &file.header);

    let mut builder = TableBuilder::new();
    builder.add_col_l("type", "the segment type");
    builder.add_col_r(
        "offset",
        "the offset into the ELF file at which the segment appears",
    );
    if file.is_core() {
        builder.add_col_r("vaddr", "the virtual address the segment starts at");
    }
    builder.add_col_r("file size", "the size of the segment on disk");
    if file.is_core() {
        builder.add_col_r("memory size", "the size of the segment in memory");
    }
    builder.add_col_r("flags", "executable, writeable, and/or readable");

    for segment in segments.iter() {
        add_field!(builder, "type", "{:?}", segment.stype);
        add_field!(builder, "offset", "{:x}", segment.offset);
        if file.is_core() {
            add_field!(builder, "vaddr", "{:x}", segment.vaddr);
        }
        add_field!(builder, "file size", "{:x}", segment.file_size);
        if file.is_core() {
            add_field!(builder, "memory size", "{:x}", segment.mem_size);
        }
        add_field!(builder, "flags", "{}", ProgramHeader::flags(segment.flags));
    }

    builder.println(args.titles, args.explain);
    if args.explain {
        println!();
        println!("Numeric fields are all in hex. Usually it's more informative to use");
        println!("other commands like `info loads` or `info mapped`.");
    }
}

pub fn info_strings(files: &ElfFiles, args: &StringsArgs) {
    let file = get_file(files, true);
    let num_sections = file.get_sections().len();

    let mut found = false;
    for index in 0..num_sections {
        if args.index.unwrap_or(index) == index {
            let section = &file.get_sections()[index];
            if section.stype == SectionType::StringTable {
                if found {
                    println!();
                }
                println!("section {index}");
                let strings = file.find_strings(section, args.max_results);
                for (i, s) in strings.iter().enumerate() {
                    println!("{i}: {s}");
                }
                if strings.len() == args.max_results {
                    println!("...");
                }
                found = true;
            }
        }
    }
}

pub fn info_symbols(files: &ElfFiles, args: &TableArgs) {
    let mut builder = TableBuilder::new();
    builder.add_col_r("index", "symbol index");
    builder.add_col_l("name", "the symbol name");
    builder.add_col_l("type", "the symbol type");
    builder.add_col_l("dynamic", "true if the symbol is from a shared lib");
    builder.add_col_r("value", "address, absolute value, etc (in hex)");
    builder.add_col_r("size", "size of the value, 0 for unknown or undefined");
    builder.add_col_l("binding", "linkage visibility and behavior");
    builder.add_col_l(
        "visibility",
        "whether the symbol is visible outside its object file",
    );
    builder.add_col_l(
        "related",
        "indicates a related section or marks the entry as an absolute value",
    );

    // TODO double check that function pointers are legit
    // TODO sort rows? provide some sort of generic table sort support?
    // TODO maybe also filtering options, eg max-results and filter by col value
    //      two options for filter by col? or something like --filter="type=Func"?
    //      maybe also a complement option
    let file = get_file(files, args.exe);
    let tables = [file.find_dynamic_symbols(), file.find_symbols()];
    let tables = tables.iter().flatten().collect::<Vec<_>>();
    for table in tables.iter() {
        println!("using section {}", table.section.link.0);
        for (i, e) in table.entries.iter().enumerate() {
            // TODO function names can be really long (especially with name mangling)
            // readelf puts a pretty small cap on these, maybe we should default to the same
            let name = file
                .find_string(table.section.link, e.name)
                .unwrap_or("unknown".to_string());
            let name = format!("{} ({})", name, e.name.0);
            add_field!(builder, "index", i);
            add_field!(builder, "name", name);
            add_field!(builder, "dynamic", table.dynamic);
            add_field!(builder, "value", "{:x}", e.value);
            add_field!(builder, "size", e.size);
            add_field!(builder, "type", "{:?}", e.stype);
            add_field!(builder, "binding", "{:?}", e.binding);
            add_field!(builder, "visibility", "{:?}", e.visibility);
            add_field!(builder, "related", index_to_str(file, e.index));
        }
    }

    builder.println(args.titles, args.explain);
}

fn index_to_str(file: &ElfFile, index: SymbolIndex) -> String {
    match index {
        SymbolIndex::Abs => "Value".to_string(),
        SymbolIndex::Common => "Common".to_string(),
        SymbolIndex::Index(i) => file
            .find_section_name(i)
            .unwrap_or("bad section index".to_string()),
        SymbolIndex::Undef => "".to_string(),
        SymbolIndex::XIndex => "not implemented".to_string(), // TODO
    }
}
