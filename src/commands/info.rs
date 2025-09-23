use super::tables::{add_field, add_simple};
use crate::commands::tables::{SimpleTableBuilder, TableBuilder};
use crate::elf::VirtualAddr;
use crate::repl::{ExplainArgs, LineArgs, RegistersArgs};
use crate::utils;
use crate::utils::Styling;
use crate::{elf::ElfFile, elf::ElfFiles, repl::TableArgs};
use std::cmp::Ordering;

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

pub fn info_line(files: &ElfFiles, args: &LineArgs) {
    match files.find_line(VirtualAddr(args.addr)) {
        Ok((file, line, col)) => println!("{file}:{line}:{col}"),
        Err(e) => println!("{e}"),
    }
}

pub fn info_mapped(files: &ElfFiles, args: &TableArgs) {
    let file = get_file(files, args.exe);
    if let Some(files) = file.get_memory_mapped_files() {
        let mut builder = TableBuilder::new();
        builder.add_col_l(
            "start",
            "the virtual address for the first byte the file is mapped into",
        );
        builder.add_col_r(
            "end",
            "the virtual address after the last byte the file is mapped into",
        );
        builder.add_col_r("size", "the size of the file in memory (decimal)");
        builder.add_col_l("file name", "path to the file");

        for file in files {
            add_field!(builder, "start", "{:x}", file.vbytes.start.0);
            add_field!(builder, "end", "{:x}", file.vbytes.end().0);
            add_field!(builder, "size", file.vbytes.size);
            add_field!(builder, "file name", file.file_name);
        }

        builder.println(args.titles, args.explain);
    } else {
        println!("No memory mapped files found.");
    }
}

pub fn info_process(files: &ElfFiles, args: &ExplainArgs) {
    let file = get_file(files, args.exe);
    if let Some(status) = file.find_prstatus() {
        let mut b = SimpleTableBuilder::new();

        add_simple!(
            b,
            "pid",
            status.pid,
            "the process id for the exe that produced the file"
        );
        add_simple!(
            b,
            "file",
            file.path.display(),
            "path to the ELF file that was loaded"
        );

        b.println(args.explain);
    } else {
        println!("No prstatus found");
    }
}

pub fn info_registers(files: &ElfFiles, args: &RegistersArgs) {
    // These come out in a really annoying order so we'll sort them.
    let file = get_file(files, args.exe);
    if let Some(status) = file.find_prstatus() {
        let mut tuples: Vec<(&'static str, u64)> = status
            .registers
            .iter()
            .enumerate()
            .filter_map(|(i, value)| {
                if args.all || !status.is_rare_register(i) {
                    let name = status.register_name(i);
                    if name != "?" {
                        return Some((name, *value));
                    }
                }
                None
            })
            .collect();

        tuples.sort_by(|lhs, rhs| {
            let lhs_num = lhs.0[1..].parse::<i32>();
            let rhs_num = rhs.0[1..].parse::<i32>();
            if let Ok(n1) = lhs_num
                && let Ok(n2) = rhs_num
            {
                // numeric registers are sorted by value, eg r9 before r11
                n1.cmp(&n2)
            } else if lhs_num.is_ok() {
                // alpha registers appear before numeric, eg rbp before r10
                Ordering::Greater
            } else if rhs_num.is_ok() {
                // alpha registers appear before numeric, eg rbp before r10
                Ordering::Less
            } else {
                // alpha registers are sorted as is, eg rbp before rip
                lhs.cmp(rhs)
            }
        });

        let mut builder = TableBuilder::new();
        builder.add_col_l("name", "the register name");
        builder.add_col_r("hex", "the register value in hex");
        builder.add_col_r("decimal", "the register value in decimal");

        for (name, value) in tuples.iter() {
            add_field!(builder, "name", name);
            add_field!(builder, "hex", "{:x}", value);
            add_field!(builder, "decimal", value);
        }

        builder.println(args.titles, args.explain);

        if args.explain {
            // TODO really these are x86 only
            utils::explain(
                "rip",
                "points to the instruction pointer currently being executed",
            );
            utils::explain(
                "rsp",
                "points to the bottom of the stack, local variables appear after this",
            );
            utils::explain(
                "rbp",
                "points to the top of the stack (depending on compiler options)",
            );
        }
    } else {
        println!("No prstatus found");
    }
}

pub fn info_signals(files: &ElfFiles, args: &TableArgs) {
    let file = get_file(files, args.exe);
    let maybe_status = file.find_prstatus();
    let maybe_signal = file.find_signal_info();

    if let Some(status) = &maybe_status {
        println!("{}", status.signal()); // this one does a nice job formatting signal and code
    } else {
        utils::warn("Couldn't find prstatus note");
    }

    if let Some(info) = &maybe_signal {
        let mut b = SimpleTableBuilder::new();
        match &info.details {
            crate::elf::SignalDetails::Fault(details) => {
                add_simple!(
                    b,
                    "fault addr",
                    "0x{:x}",
                    details.fault_addr,
                    "the address that caused the core"
                );
            }
            crate::elf::SignalDetails::Kill(details) => {
                add_simple!(
                    b,
                    "sender pid",
                    details.sender_pid,
                    "the pid of the process that sent the signal"
                );
                add_simple!(
                    b,
                    "sender uid",
                    details.sender_uid,
                    "the uid of the process that sent the signal"
                );
            }
            crate::elf::SignalDetails::Posix(details) => {
                add_simple!(
                    b,
                    "sender pid",
                    details.sender_pid,
                    "the pid of the process that sent the signal"
                );
                add_simple!(
                    b,
                    "sender uid",
                    details.sender_uid,
                    "the uid of the process that sent the signal"
                );
            }
            crate::elf::SignalDetails::Child(details) => {
                add_simple!(
                    b,
                    "child_pid",
                    details.child_pid,
                    "pid of the child process"
                );
                add_simple!(
                    b,
                    "child_uid",
                    details.child_uid,
                    "uid of the child process"
                );
                add_simple!(
                    b,
                    "exit_code",
                    details.exit_code,
                    "exit code of the child process"
                );
            }
            _ => (),
        }
        b.println(args.explain);
    } else {
        utils::warn("Couldn't find signal note");
    }
}
