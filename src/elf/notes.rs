//! The various notes in an ELF file. These provide information about the process and
//! not all may be present.
use super::Stream;
use crate::utils;
use std::error::Error;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum NoteType {
    /// The AuxV note is AT_SYSINFO_EHDR which contains a small shared library
    /// mapped into the address space of all user-space applications. It's used
    /// to speed up calling common kernel functions. See https://man7.org/linux/man-pages/man7/vdso.7.html
    AuxV,

    /// Memory-mapped files, see fill_files_note in https://android.googlesource.com/kernel/common/+/6e7bfa046de8/fs/binfmt_elf.c
    File,

    /// Signal info, pid, etc. See elf_prstatus in https://docs.huihoo.com/doxygen/linux/kernel/3.7/uapi_2linux_2elfcore_8h_source.html.
    PrStatus,

    /// Floating point register values.
    PrFPReg,

    /// Process state info, e.g. whether it's running, sleeping, or a zombie. Also the
    /// name and arguments for the executable. See elf_prpsinfo in https://docs.huihoo.com/doxygen/linux/kernel/3.7/uapi_2linux_2elfcore_8h_source.html
    PrPsInfo, // TODO expose some of this

    /// Seen this documented as a elf_siginfo which seems silly because PrStatus has that.
    /// It's also 80 bytes which is much larger than that so I think it may be a siginfo_t
    /// which has the usual signal stuff plus the fault address. TODO update this comment
    SigInfo,
}

impl NoteType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(NoteType::PrStatus), // see https://docs.huihoo.com/doxygen/linux/kernel/3.7/include_2uapi_2linux_2elf_8h_source.html
            2 => Some(NoteType::PrFPReg),
            3 => Some(NoteType::PrPsInfo),
            6 => Some(NoteType::AuxV),
            0x53494749 => Some(NoteType::SigInfo),
            0x46494c45 => Some(NoteType::File),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct NoteContents {
    pub offset: usize,
    pub size: u32,
}

pub struct PrStatus {
    /// The signal that terminated the process.
    pub signal_num: i32,

    /// Further details about the signal. For example, code can be SEGV_MAPERR (bad
    /// address) or SEGV_ACCERR (bad permessions) for the SIGSEGV signal. See
    /// https://www.mkssoftware.com/docs/man5/siginfo_t.5.asp#Signal_Codes for more.
    pub signal_code: i32,

    /// If non-zero, the errno associated with the signal.
    pub errno: i32,

    /// The process ID of the process that generated this core file.
    pub pid: i32,

    /// General purpose rehisters. For arm and x86 they are laid out as in pt_regs
    /// in https://elixir.bootlin.com/linux/v4.9/source/arch/x86/include/uapi/asm/ptrace.h#L60
    pub registers: Vec<u64>,
}

/// Similar to the signal info in PrStatus but with additional details.
pub struct SigInfo {
    /// The signal that terminated the process.
    pub signal_num: i32,

    /// If non-zero, the errno associated with the signal.
    pub errno: i32,

    /// Further details about the signal. For example, code can be SEGV_MAPERR (bad
    /// address) or SEGV_ACCERR (bad permessions) for the SIGSEGV signal. See
    /// https://www.mkssoftware.com/docs/man5/siginfo_t.5.asp#Signal_Codes for more.
    /// TODO is this right?
    pub signal_code: i32,

    /// Information associated with the specific signal that killed the process.
    pub details: SignalDetails,
}

pub enum SignalDetails {
    Child(ChildSignal),
    Fault(FaultSignal),
    Kill(KillSignal),
    MesgQ,
    Poll,
    Posix(PosixSignal),
    Sys,
    Timer,
}

pub struct ChildSignal {
    pub child_pid: i32,
    pub child_uid: i32,
    pub exit_code: i32,
}

pub struct FaultSignal {
    pub fault_addr: u64,
}

pub struct KillSignal {
    pub sender_pid: i32,
    pub sender_uid: i32,
}

pub struct PosixSignal {
    pub sender_pid: i32,
    pub sender_uid: i32,
}

impl PrStatus {
    pub fn signal(&self) -> &'static str {
        match self.signal_num {
            1 => "SIGHUP", // see https://man7.org/linux/man-pages/man7/signal.7.html
            2 => "SIGINT",
            3 => "SIGQUIT",
            4 => match self.signal_code {
                // and https://sites.uclouvain.be/SystInfo/usr/include/bits/siginfo.h.html
                1 => "SIGILL: Illegal opcode",          // ILL_ILLOPC
                2 => "SIGILL: Illegal operand",         // ILL_ILLOPN
                3 => "SIGILL: Illegal addressing mode", // ILL_ILLADR
                4 => "SIGILL: Illegal trap",            // ILL_ILLTRP
                5 => "SIGILL: Privileged opcode",       // ILL_PRVOPC
                6 => "SIGILL: Privileged register",     // ILL_PRVREG
                7 => "SIGILL: Coprocessor error",       // ILL_COPROC
                8 => "SIGILL: Internal stack error",    // ILL_BADSTK
                _ => "SIGILL",
            },
            5 => match self.signal_code {
                1 => "SIGTRAP: Process breakpoint", // TRAP_BRKPT
                2 => "SIGTRAP: Process trace trap", // TRAP_TRACE
                _ => "SIGTRAP",
            },
            6 => "SIGABRT",
            7 => match self.signal_code {
                1 => "SIGBUS: Invalid address alignment",      // BUS_ADRALN
                2 => "SIGBUS: Non-existant physical address",  // BUS_ADRERR
                3 => "SIGBUS: Object specific hardware error", // BUS_OBJERR
                _ => "SIGBUS",
            },
            8 => match self.signal_code {
                1 => "SIGFPE: Integer divide by zero",           // FPE_INTDIV
                2 => "SIGFPE: Integer overflow",                 // FPE_INTOVF
                3 => "SIGFPE: Floating point divide by zero",    // FPE_FLTDIV
                4 => "SIGFPE: Floating point overflow",          // FPE_FLTOVF
                5 => "SIGFPE: Floating point underflow",         // FPE_FLTUND
                6 => "SIGFPE: Floating point inexact result",    // FPE_FLTRES
                7 => "SIGFPE: Floating point invalid operation", // FPE_FLTINV
                8 => "SIGFPE: Subscript out of range",           // FPE_FLTSUB
                _ => "SIGFPE",
            },
            9 => "SIGKILL",
            10 => "SIGUSR1",
            11 => match self.signal_code {
                1 => "SIGSEGV: Address not mapped to object", // SEGV_MAPERR
                2 => "SIGSEGV: Invalid permissions for mapped object", // SEGV_ACCERR
                _ => "SIGSEGV",
            },
            12 => "SIGUSR2",
            13 => "SIGPIPE",
            14 => "SIGALRM",
            15 => "SIGTERM",
            16 => "SIGSTKFLT",
            17 => match self.signal_code {
                1 => "SIGCHLD: Child has exited",            // CLD_EXITED
                2 => "SIGCHLD: Child was killed",            // CLD_KILLED
                3 => "SIGCHLD: Child terminated abnormally", // CLD_DUMPED
                4 => "SIGCHLD: Traced child has trapped",    // CLD_TRAPPED
                5 => "SIGCHLD: Child has stopped",           // CLD_STOPPED
                6 => "SIGCHLD: Stopped child has continued", // CLD_CONTINUED
                _ => "SIGCHLD",
            },
            18 => "SIGCONT",
            19 => "SIGSTOP",
            20 => "SIGTSTP",
            21 => "SIGTTIN",
            22 => "SIGTTOU",
            23 => "SIGURG",
            24 => "SIGXCPU",
            25 => "SIGXFSZ",
            26 => "SIGVTALRM",
            27 => "SIGPROF",
            28 => "SIGWINCH",
            29 => "SIGIO",
            30 => "SIGPWR",
            31 => "SIGSYS",
            _ => "unknown signal",
        }
    }

    /// Returns the instruction address within the currently executing function.
    pub fn get_ip(&self) -> u64 {
        self.registers[16]
    }

    /// Points to after the end of locals on the stack and contains the callers stack top
    /// (rbp). Returns garbage if -fomit-frame-pointer is used or for optimized builds
    /// (when -fno-omit-frame-pointer isn't set).
    pub fn get_frame_stack_top(&self) -> u64 {
        self.registers[4]
    }

    /// Points to the start of locals on the stack (rsp). Debug info has to be used to
    /// figure out the amount of space locals take.
    pub fn get_frame_stack_bottom(&self) -> u64 {
        self.registers[19]
    }

    /// Returns true for stuff like segment registers.
    pub fn is_rare_register(&self, n: usize) -> bool {
        match n {
            // TODO: good only for x86(?) and arm
            17 => true, // cs
            18 => true, // eflags
            20 => true, // ss
            22 => true, // ds
            23 => true, // es
            24 => true, // fs
            25 => true, // gs
            _ => false,
        }
    }

    pub fn register_name(&self, n: usize) -> &'static str {
        match n {
            // TODO: good only for x86(?) and arm
            0 => "r15",
            1 => "r14",
            2 => "r13",
            3 => "r12",
            4 => "rbp",
            5 => "rbx",
            6 => "r11",
            7 => "r10",
            8 => "r9",
            9 => "r8",
            10 => "rax",
            11 => "rcx",
            12 => "rdx",
            13 => "rsi",
            14 => "rdi",
            16 => "rip",
            17 => "cs",
            18 => "eflags",
            19 => "rsp",
            20 => "ss",
            22 => "ds", // TODO not sure these last few are correct
            23 => "es",
            24 => "fs",
            25 => "gs",
            _ => "?",
        }
    }
}

pub struct MemoryMappedFile {
    /// Address of the start of the file in memory.
    pub start_addr: u64,

    /// Address of the end of the file in memory.
    pub end_addr: u64,

    /// Offset into the file used when memory mapping.
    pub offset: u64,

    /// The name of the file.
    pub file_name: String,
}

pub fn read_note(s: &mut Stream) -> Result<(String, u32, NoteContents), Box<dyn Error>> {
    let n_namesz = s.read_word()?;
    let n_descsz = s.read_word()?;
    let n_type = s.read_word()?;

    let name_bytes = s.reader.slice(s.offset, (n_namesz - 1) as usize)?.to_vec();
    let name = String::from_utf8(name_bytes)?;
    s.offset += utils::align_to_word(n_namesz) as usize; // align desc to 4-byte boundary

    let desc_offset = s.offset;
    s.offset += utils::align_to_word(n_descsz) as usize; // align next note to 4-byte boundary

    Ok((
        name,
        n_type,
        NoteContents {
            offset: desc_offset,
            size: n_descsz,
        },
    ))
}
