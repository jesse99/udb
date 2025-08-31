use crate::elf::LoadSegment;
use crate::repl::HexdumpOffsets;
use crate::{
    elf::{ElfFile, Reader},
    repl::{FindArgs, HexdumpArgs},
    utils::warn,
};
use std::error::Error;

/// Returns pointers to the instructions within the functions in the current call chain.
fn raw_backtrace(core: &ElfFile) -> Result<Vec<u64>, Box<dyn Error>> {
    // TODO move this into debug module
    // see https://eli.thegreenplace.net/2011/09/06/stack-frame-layout-on-x86-64
    let mut bt = Vec::new();
    if let Some(status) = core.find_prstatus() {
        let addr = status.get_ip();
        bt.push(addr);

        let mut rbp = status.get_frame_stack_top(); // TODO won't work for release
        if let Some(load) = core.find_load_segment(rbp)
            && load.writeable()
        {
            // we expect stack to be within one segment
            // TODO could do some validation here but I think we want to be fairly permissive
            while load.vaddr <= rbp && rbp < load.vaddr + load.size {
                let delta = rbp - load.vaddr;
                rbp = core
                    .reader
                    .read_xword((load.offset + delta) as usize)
                    .unwrap();

                let addr = core
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

pub fn backtrace(core: &ElfFile) {
    match raw_backtrace(&core) {
        Ok(bt) => bt.iter().for_each(|a| println!("{a:x}")),
        Err(e) => println!("{e}"),
    }
}

pub fn find(core: &ElfFile, args: &FindArgs) {
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
                                addr: i as u64 + load.vaddr,
                                count: args.count,
                                offsets: HexdumpOffsets::None,
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

    fn search_all(core: &ElfFile, args: &FindArgs, bytes: &Vec<u8>) {
        let mut count = 0;
        let mut offset = 0;
        let mut offsets = Vec::new(); // we'll print addresses first

        let mut found_addr = false;
        while offset + bytes.len() < core.reader.len() {
            if match_bytes(&core.reader, offset, bytes) {
                match core.find_vaddr(offset as u64) {
                    Some((load, addr)) => {
                        if !found_addr {
                            println!("Addresses:");
                            found_addr = true;
                        }
                        println!("   0x{:x}", addr);

                        if args.count > 0 {
                            print!("   ");
                            hexdump_segment(
                                core,
                                &HexdumpArgs {
                                    addr,
                                    count: args.count,
                                    offsets: HexdumpOffsets::None,
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
            println!("Offsets:");
            for offset in offsets.iter() {
                println!("   0x{:x}", offset);

                if args.count > 0 {
                    print!("   ");
                    core.reader
                        .hex_dump(0, *offset, args.count, HexdumpOffsets::None);
                    println!();
                }
            }
        }
    }

    fn find(core: &ElfFile, args: &FindArgs, bytes: &Vec<u8>) {
        if args.all {
            search_all(core, args, bytes);
        } else {
            search_load_segments(core, args, bytes);
        }
    }

    // TODO there are probably crates with better substring algorithms
    // TODO might also help to read words at a time where possible
    if let Some(s) = &args.hex {
        // TODO need to search both the exe and core (for --all)
        match byte_str_to_vec(s) {
            Ok(bytes) => find(core, args, &bytes),
            Err(err) => warn(&err.to_string()),
        }
    } else if let Some(s) = &args.string {
        let bytes = ascii_str_to_vec(s);
        find(core, args, &bytes);
    }
}

pub fn hexdump(core: &ElfFile, args: &HexdumpArgs) {
    if let Some(load) = core.find_load_segment(args.addr) {
        hexdump_segment(core, args, load);
    } else {
        warn(
            "couldn't find a load segment: treating addr as an offset from the start of the ELF file",
        );
        hexdump_any(core, args.addr as usize, args.count);
    }
}

pub fn hexdump_segment(core: &ElfFile, args: &HexdumpArgs, load: &LoadSegment) {
    let delta = args.addr - load.vaddr;
    let offset = load.offset + delta; // all zeros
    core.reader
        .hex_dump(args.addr, offset as usize, args.count, args.offsets);
}

pub fn hexdump_any(core: &ElfFile, offset: usize, count: usize) {
    core.reader.hex_dump(0, offset, count, HexdumpOffsets::Zero);
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
