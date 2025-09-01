use crate::elf::{ElfFile, LoadSegment};
use crate::repl::HexdumpLabels;
use crate::{
    elf::{ElfFiles, Reader},
    repl::{FindArgs, HexdumpArgs},
    utils,
};
use std::error::Error;

/// Returns pointers to the instructions within the functions in the current call chain.
fn raw_backtrace(files: &ElfFiles) -> Result<Vec<u64>, Box<dyn Error>> {
    // TODO move this into debug module
    // see https://eli.thegreenplace.net/2011/09/06/stack-frame-layout-on-x86-64
    let mut bt = Vec::new();
    if let Some(status) = files.find_prstatus() {
        let addr = status.get_ip();
        bt.push(addr);

        let mut rbp = status.get_frame_stack_top(); // TODO won't work for release
        if let Some(load) = files.find_load_segment(rbp)
            && load.writeable()
        {
            // we expect stack to be within one segment
            // TODO could do some validation here but I think we want to be fairly permissive
            while load.vaddr <= rbp && rbp < load.vaddr + load.size {
                let delta = rbp - load.vaddr;
                rbp = files
                    .core
                    .as_ref()
                    .unwrap() // safe because find_prstatus worked
                    .reader
                    .read_xword((load.offset + delta) as usize)
                    .unwrap();

                let addr = files
                    .core
                    .as_ref()
                    .unwrap()
                    .reader
                    .read_xword((load.offset + delta + 8) as usize)
                    .unwrap();
                bt.push(addr);
            }
        } else {
            return Err("Couldn't find load segment".into());
        }
    } else {
        return Err("Couldn't find prstatus".into());
    }
    Ok(bt)
}

pub fn backtrace(files: &ElfFiles) {
    match raw_backtrace(files) {
        Ok(bt) => bt.iter().for_each(|a| println!("{a:x}")),
        Err(e) => println!("{e}"),
    }
}

pub fn find(files: &ElfFiles, args: &FindArgs) {
    fn match_bytes(reader: &Reader, i: usize, bytes: &Vec<u8>) -> bool {
        for (j, byte) in bytes.iter().enumerate() {
            match reader.read_byte(i + j) {
                Ok(b) => {
                    if b != *byte {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }
        true
    }

    fn search_load_segments(core: &ElfFile, args: &FindArgs, bytes: &Vec<u8>) {
        let mut count = 0;
        for load in core.loads.iter() {
            let mut i = 0;
            while i + bytes.len() < load.size as usize {
                if match_bytes(&core.reader, i + load.offset as usize, bytes) {
                    println!("0x{:x}", i + load.vaddr as usize);
                    if args.count > 0 {
                        hexdump_segment(
                            core,
                            &HexdumpArgs {
                                value: i as u64 + load.vaddr,
                                offset: false,
                                count: args.count,
                                labels: HexdumpLabels::None,
                                exe: false,
                            },
                            load,
                        );
                        println!();
                    }
                    i += bytes.len();
                    count += 1;
                    if count == args.max_results {
                        println!("...");
                        return;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    fn search_all(prefix: &str, file: &ElfFile, args: &FindArgs, bytes: &Vec<u8>) {
        let mut count = 0;
        let mut offset = 0;
        let mut offsets = Vec::new(); // we'll print addresses first

        let mut found_addr = false;
        while offset + bytes.len() < file.reader.len() {
            if match_bytes(&file.reader, offset, bytes) {
                match file.find_vaddr(offset as u64) {
                    Some((load, addr)) => {
                        if !found_addr {
                            println!("{prefix}Addresses:");
                            found_addr = true;
                        }
                        println!("   0x{:x}", addr);

                        if args.count > 0 {
                            print!("   ");
                            hexdump_segment(
                                file,
                                &HexdumpArgs {
                                    value: addr,
                                    offset: false,
                                    exe: false,
                                    count: args.count,
                                    labels: HexdumpLabels::None,
                                },
                                load,
                            );
                            println!();
                        }
                    }
                    None => offsets.push(offset),
                }
                offset += bytes.len();
                count += 1;
                if count == args.max_results {
                    println!("...");
                    return;
                }
            } else {
                offset += 1;
            }
        }

        if !offsets.is_empty() {
            println!("{prefix}Offsets:");
            for offset in offsets.iter() {
                println!("   0x{:x}", offset);

                if args.count > 0 {
                    print!("   ");
                    file.reader
                        .hex_dump(0, *offset, args.count, HexdumpLabels::None);
                    println!();
                }
            }
        }
    }

    fn find(files: &ElfFiles, args: &FindArgs, bytes: &Vec<u8>) {
        if args.all {
            if let Some(core) = &files.core
                && let Some(exe) = &files.exe
            {
                search_all("Core ", core, args, bytes);
                search_all("Exe ", exe, args, bytes);
            } else if let Some(core) = &files.core {
                search_all("", core, args, bytes);
            } else {
                search_all("", files.exe.as_ref().unwrap(), args, bytes); // safe because we'll always have either core or exe
            }
        } else {
            if let Some(core) = &files.core {
                search_load_segments(core, args, bytes);
            } else {
                // Technically we should only do this if --all is used but it's kind of
                // silly to not do a search if all we have is an exe.
                search_all("", files.exe.as_ref().unwrap(), args, bytes);
            }
        }
    }

    // TODO there are probably crates with better substring algorithms
    // TODO might also help to read words at a time where possible
    if let Some(s) = &args.hex {
        // TODO need to search both the exe and core (for --all)
        match byte_str_to_vec(s) {
            Ok(bytes) => find(files, args, &bytes),
            Err(err) => utils::warn(&err.to_string()),
        }
    } else if let Some(s) = &args.string {
        let bytes = ascii_str_to_vec(s);
        find(files, args, &bytes);
    }
}

pub fn hexdump(files: &ElfFiles, args: &HexdumpArgs) {
    if args.offset {
        if args.exe {
            match &files.exe {
                Some(file) => hexdump_any(file, args.value as usize, args.count),
                None => utils::warn("--exe was used but there is no exe"),
            }
        } else {
            match &files.core {
                Some(file) => hexdump_any(file, args.value as usize, args.count),
                None => hexdump_any(files.exe.as_ref().unwrap(), args.value as usize, args.count),
            }
        }
    } else if args.exe {
        match &files.exe {
            Some(file) => match file.find_load_segment(args.value) {
                Some(load) => hexdump_segment(file, args, load),
                None => utils::warn("--couldn't find a load segment for the address"),
            },
            None => utils::warn("--exe was used but there is no exe"),
        }
    } else {
        match &files.core {
            Some(file) => match file.find_load_segment(args.value) {
                Some(load) => hexdump_segment(file, args, load),
                None => utils::warn("couldn't find a load segment for the address"),
            },
            None => {
                let file = files.exe.as_ref().unwrap();
                match file.find_load_segment(args.value) {
                    Some(load) => hexdump_segment(file, args, load),
                    None => utils::warn("couldn't find a load segment for the address"),
                }
            }
        }
    }
}

pub fn hexdump_segment(file: &ElfFile, args: &HexdumpArgs, load: &LoadSegment) {
    let delta = args.value - load.vaddr;
    let offset = load.offset + delta; // all zeros
    file.reader
        .hex_dump(args.value, offset as usize, args.count, args.labels);
}

pub fn hexdump_any(file: &ElfFile, offset: usize, count: usize) {
    file.reader.hex_dump(0, offset, count, HexdumpLabels::Zero);
}

fn ascii_str_to_vec(str: &str) -> Vec<u8> {
    let mut result = Vec::new();

    for ch in str.chars() {
        let mut buffer = [0; 4]; // 4 is always large enough
        let n = ch.encode_utf8(&mut buffer).len();
        result.extend(buffer.iter().take(n));
    }

    result
}

fn byte_str_to_vec(str: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = Vec::new();

    let mut i = 0;
    let chars: Vec<char> = str.chars().collect();
    while i < chars.len() {
        if chars[i] == ' ' {
            // ignore spaces
            i += 1;
        } else if i + 1 < chars.len()
            && chars[i].is_ascii_hexdigit()
            && chars[i + 1].is_ascii_hexdigit()
        {
            let s = format!("{}{}", chars[i], chars[i + 1]);
            let byte = u8::from_str_radix(&s, 16).unwrap();
            result.push(byte);
            i += 2;
        } else {
            return Err("Expected a string of hex bytes with optional spaces between bytes".into());
        }
    }

    Ok(result)
}
