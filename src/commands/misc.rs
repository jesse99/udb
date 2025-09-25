use crate::elf::{ElfFile, LoadSegment, Offset, VirtualAddr};
use crate::repl::HexdumpLabels;
use crate::utils::{uwrite, uwriteln};
use crate::{
    elf::{ElfFiles, Reader},
    repl::{FindArgs, HexdumpArgs},
    utils,
};
use std::error::Error;
use std::io::Write;

/// Returns pointers to the instructions within the functions in the current call chain.
fn raw_backtrace(files: &ElfFiles) -> Result<Vec<VirtualAddr>, Box<dyn Error>> {
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
            while let Some(offset) = load.to_offset(rbp) {
                rbp = VirtualAddr::from_raw(
                    files
                        .core
                        .as_ref()
                        .unwrap() // safe because find_prstatus worked
                        .reader
                        .read_xword(offset)
                        .unwrap(),
                );

                let addr = VirtualAddr::from_raw(
                    files
                        .core
                        .as_ref()
                        .unwrap()
                        .reader
                        .read_xword(offset + 8)
                        .unwrap(),
                );
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

pub fn backtrace(mut out: impl Write, files: &ElfFiles) {
    match raw_backtrace(files) {
        Ok(bt) => bt.iter().for_each(|a| match files.find_line(*a) {
            Ok((file, line, col)) => uwriteln!(out, "0x{:x} {file}:{line}:{col}", a.0),
            Err(_) => uwriteln!(out, "0x{:x}", a.0),
        }),
        Err(e) => uwriteln!(out, "{e}"),
    }
}

pub fn find(out: impl Write, files: &ElfFiles, args: &FindArgs) {
    fn match_bytes(reader: &Reader, i: usize, bytes: &[u8]) -> bool {
        for (j, byte) in bytes.iter().enumerate() {
            let offset = Offset::from_raw((i + j) as u64);
            match reader.read_byte(offset) {
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

    fn search_load_segments(mut out: impl Write, core: &ElfFile, args: &FindArgs, bytes: &[u8]) {
        let mut count = 0;
        for load in core.loads.iter() {
            let mut i = 0;
            while i + bytes.len() < load.obytes.size {
                if match_bytes(core.reader, i + load.obytes.start.0 as usize, bytes) {
                    uwriteln!(out, "0x{:x}", i + load.vbytes.start.0 as usize);
                    if args.count > 0 {
                        hexdump_segment(
                            &mut out,
                            core,
                            &HexdumpArgs {
                                value: i as u64 + load.vbytes.start.0,
                                offset: false,
                                count: args.count,
                                labels: HexdumpLabels::None,
                                exe: false,
                            },
                            load,
                        );
                        uwriteln!(out);
                    }
                    i += bytes.len();
                    count += 1;
                    if count == args.max_results {
                        uwriteln!(out, "...");
                        return;
                    }
                } else {
                    i += 1;
                }
            }
        }
    }

    fn search_all(
        out: &mut impl Write,
        prefix: &str,
        file: &ElfFile,
        args: &FindArgs,
        bytes: &[u8],
    ) {
        let mut count = 0;
        let mut offset = Offset::from_raw(0);
        let mut offsets = Vec::new(); // we'll print addresses first

        let mut found_addr = false;
        while offset.0 as usize + bytes.len() < file.reader.len() {
            if match_bytes(file.reader, offset.0 as usize, bytes) {
                match file.offset_to_vaddr(offset) {
                    Some((load, addr)) => {
                        if !found_addr {
                            uwriteln!(out, "{prefix}Addresses:");
                            found_addr = true;
                        }
                        uwriteln!(out, "   0x{:x}", addr.0);

                        if args.count > 0 {
                            uwrite!(out, "   ");
                            hexdump_segment(
                                out,
                                file,
                                &HexdumpArgs {
                                    value: addr.0,
                                    offset: false,
                                    exe: false,
                                    count: args.count,
                                    labels: HexdumpLabels::None,
                                },
                                load,
                            );
                            uwriteln!(out);
                        }
                        count += 1;
                        if count == args.max_results {
                            uwriteln!(out, "   ...");
                            return;
                        }
                    }
                    None => offsets.push(offset), // we'll print these later
                }
                offset = offset + bytes.len() as i64;
            } else {
                offset = offset + 1;
            }
        }

        if !offsets.is_empty() {
            count = 0;
            uwriteln!(out, "{prefix}Offsets:");
            for offset in offsets.iter() {
                uwriteln!(out, "   0x{:x}", offset.0);

                if args.count > 0 {
                    uwrite!(out, "   ");
                    file.reader
                        .hex_dump(out, 0, *offset, args.count, HexdumpLabels::None);
                    uwriteln!(out);
                }
                count += 1;
                if count == args.max_results {
                    uwriteln!(out, "   ...");
                    return;
                }
            }
        }
    }

    fn find(mut out: impl Write, files: &ElfFiles, args: &FindArgs, bytes: &[u8]) {
        if args.all {
            if let Some(core) = &files.core
                && let Some(exe) = &files.exe
            {
                search_all(&mut out, "Core ", core, args, bytes);
                search_all(&mut out, "Exe ", exe, args, bytes);
            } else if let Some(core) = &files.core {
                search_all(&mut out, "", core, args, bytes);
            } else {
                search_all(&mut out, "", files.exe.as_ref().unwrap(), args, bytes); // safe because we'll always have either core or exe
            }
        } else if let Some(core) = &files.core {
            search_load_segments(out, core, args, bytes);
        } else {
            // Technically we should only do this if --all is used but it's kind of
            // silly to not do a search if all we have is an exe.
            search_all(&mut out, "", files.exe.as_ref().unwrap(), args, bytes);
        }
    }

    // TODO there are probably crates with better substring algorithms
    // TODO might also help to read words at a time where possible
    if let Some(s) = &args.hex {
        match byte_str_to_vec(s) {
            Ok(bytes) => find(out, files, args, &bytes),
            Err(err) => utils::warn(&err.to_string()),
        }
    } else if let Some(s) = &args.string {
        let bytes = ascii_str_to_vec(s);
        find(out, files, args, &bytes);
    }
}

pub fn hexdump(mut out: impl Write, files: &ElfFiles, args: &HexdumpArgs) {
    if args.offset {
        if args.exe {
            match &files.exe {
                Some(file) => hexdump_any(out, file, Offset(args.value), args.count, args.labels),
                None => utils::warn("--exe was used but there is no exe"),
            }
        } else {
            match &files.core {
                Some(file) => hexdump_any(out, file, Offset(args.value), args.count, args.labels),
                None => hexdump_any(
                    out,
                    files.exe.as_ref().unwrap(),
                    Offset(args.value),
                    args.count,
                    args.labels,
                ),
            }
        }
    } else {
        let vaddr = VirtualAddr::from_raw(args.value);
        if args.exe {
            match &files.exe {
                Some(file) => match file.find_load_segment(vaddr) {
                    Some(load) => hexdump_segment(&mut out, file, args, load),
                    None => utils::warn("--couldn't find a load segment for the address"),
                },
                None => utils::warn("--exe was used but there is no exe"),
            }
        } else {
            match &files.core {
                Some(file) => match file.find_load_segment(vaddr) {
                    Some(load) => hexdump_segment(&mut out, file, args, load),
                    None => utils::warn("couldn't find a load segment for the address"),
                },
                None => {
                    let file = files.exe.as_ref().unwrap();
                    match file.find_load_segment(vaddr) {
                        Some(load) => hexdump_segment(&mut out, file, args, load),
                        None => utils::warn("couldn't find a load segment for the address"),
                    }
                }
            }
        }
    }
}

pub fn hexdump_segment(
    out: &mut impl Write,
    file: &ElfFile,
    args: &HexdumpArgs,
    load: &LoadSegment,
) {
    let vaddr = VirtualAddr::from_raw(args.value);
    if let Some(offset) = load.to_offset(vaddr) {
        file.reader
            .hex_dump(out, args.value, offset, args.count, args.labels);
    }
}

pub fn hexdump_any(
    mut out: impl Write,
    file: &ElfFile,
    offset: Offset,
    count: usize,
    labels: HexdumpLabels,
) {
    if labels == HexdumpLabels::Addr {
        utils::warn("Can't use --labels=address when dumping by offset");
    } else {
        file.reader.hex_dump(&mut out, 0, offset, count, labels);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{debug_results, do_test, release_results};

    #[test]
    fn bt() {
        do_test!(backtrace, debug_only); // TODO get bt working in release
    }

    #[test]
    fn find_default_str() {
        let args = FindArgs {
            all: false,
            string: Some("apple".to_string()),
            count: 0,
            hex: None,
            max_results: 0,
        };
        do_test!(find, &args);
    }

    #[test]
    fn find_all_str() {
        let args = FindArgs {
            all: true,
            string: Some("count".to_string()),
            count: 0,
            hex: None,
            max_results: 0,
        };
        do_test!(find, &args);
    }

    #[test]
    fn find_hex() {
        let args = FindArgs {
            all: false,
            string: None,
            count: 0,
            hex: Some("20".to_string()),
            max_results: 10,
        };
        do_test!(find, &args);
    }

    #[test]
    fn find_all_hex() {
        let args = FindArgs {
            all: true,
            string: None,
            count: 0,
            hex: Some("20".to_string()),
            max_results: 10,
        };
        do_test!(find, &args);
    }

    #[test]
    fn find_str_dump() {
        let args = FindArgs {
            all: false,
            string: Some("count".to_string()),
            count: 25,
            hex: None,
            max_results: 0,
        };
        do_test!(find, &args);
    }

    #[test]
    fn dump_addr() {
        let args = HexdumpArgs {
            exe: false,
            count: 16,
            labels: HexdumpLabels::None,
            offset: false,
            value: 0x7ff8fc2ceb25,
        };
        do_test!(hexdump, &args);
    }

    #[test]
    fn dump_offset() {
        let args = HexdumpArgs {
            exe: true,
            count: 32,
            labels: HexdumpLabels::Zero,
            offset: true,
            value: 0x3871,
        };
        do_test!(hexdump, &args);
    }

    #[test]
    fn dump_addr_labels() {
        let args = HexdumpArgs {
            exe: false,
            count: 34,
            labels: HexdumpLabels::Addr,
            offset: false,
            value: 0x7ff8fc2ceb25,
        };
        do_test!(hexdump, &args);
    }
}
