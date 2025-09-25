//! Definitions for the commands that are used interactively, e.g.
//! `bt` and `info registers`.
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt;

#[derive(Parser)]
#[command(version, about, long_about = None)] // TODO about?
#[command(infer_subcommands(true))] // allow abreviations
pub struct Repl {
    #[command(subcommand)]
    pub command: MainCommand,
}

#[derive(Subcommand)]
pub enum MainCommand {
    /// Show backtrace for the current thread
    Bt,

    /// Show low level information about the core and exe files
    Elf(ElfCommand),

    /// Search memory for a bit pattern
    Find(FindArgs),

    /// Show higher level information about the cored process
    Info(InfoCommand),

    /// Print memory range as hex and ascii
    Hexdump(HexdumpArgs),

    /// Exit udb
    Quit,
}

#[derive(Args)]
pub struct ElfCommand {
    #[clap(subcommand)]
    pub action: ElfAction,
}

#[derive(Args)]
pub struct InfoCommand {
    #[clap(subcommand)]
    pub action: InfoAction,
}

#[derive(Subcommand)]
pub enum ElfAction {
    /// Show ELF header
    Header(ExplainArgs),

    /// Show low level address to line tables
    Line(ElfLineArgs),

    /// Show ELF load segments
    Loads(TableArgs),

    /// Show sections
    Notes(TableArgs),

    /// Show relocations
    Relocations(TableArgs),

    /// Show sections
    Sections(TableArgs),

    /// Show segments
    Segments(TableArgs),

    /// Dump string tables
    Strings(StringsArgs),

    /// Show symbols
    Symbols(TableArgs),
}

#[derive(Subcommand)]
pub enum InfoAction {
    /// Print file and line number for a virtual address
    Line(LineArgs),

    /// Show memory mapped files
    Mapped(TableArgs),

    /// Show information the process associated with the core file
    Process(ExplainArgs),

    /// Show general purpose registers
    Registers(RegistersArgs),

    /// Show information about signals
    Signals(TableArgs),
}

#[derive(Args)]
pub struct ExplainArgs {
    /// Show core info unless there is no core or this is set
    #[arg(long)]
    pub exe: bool,

    /// Explain columns, fields, etc.
    #[arg(short, long)]
    pub explain: bool,
}

// TODO should be able to search for other stuff like ints (need to account for endian)
// TODO provide a way to restrict search area?
#[derive(Args)]
pub struct FindArgs {
    /// Default is to search virtual memory in the core file. When this is enabled all
    /// the bytes in both the exe and the core are searched.
    #[arg(long)]
    pub all: bool,

    /// Search for an UTF-8 string e.g. "the brown fox"
    #[arg(long, group = "filter")]
    pub string: Option<String>,

    /// Optionally hexdump count bytes for each address found
    #[arg(short, long, default_value_t = 0)]
    pub count: usize,

    /// Search for a hex string with spaces ignored, e.g. "ab ac acab"
    #[arg(long, group = "filter")]
    pub hex: Option<String>,

    /// Max number of results to report, 0 for unlimited
    #[arg(short, long, default_value_t = 10, requires = "filter")]
    pub max_results: usize,
}

#[derive(Args)]
pub struct ElfLineArgs {
    /// Number of lines to print in the address => line table.
    #[arg(short, long)]
    #[arg(default_value_t = 20)]
    pub max_lines: usize,
}

#[derive(Args)]
pub struct TableArgs {
    /// Show core info unless there is no core or this is set
    #[arg(long)]
    pub exe: bool,

    /// Explain columns, fields, etc.
    #[arg(short, long)]
    pub explain: bool,

    /// Add column headers
    #[arg(short, long)]
    pub titles: bool,
}

#[derive(Args)]
pub struct RegistersArgs {
    /// Also dump rarely used registers such as segment registers
    #[arg(short, long)]
    pub all: bool,

    /// Show core info unless there is no core or this is set
    #[arg(long)]
    pub exe: bool,

    /// Explain columns, fields, etc.
    #[arg(short, long)]
    pub explain: bool,

    /// Add column headers
    #[arg(short, long)]
    pub titles: bool,
}

#[derive(Args)]
pub struct LineArgs {
    /// A virtual address
    #[arg(value_parser = parse_u64_expr)]
    pub addr: u64,
}

#[derive(Args)]
pub struct HexdumpArgs {
    /// Dump the exe instead of the core file
    #[arg(long)]
    pub exe: bool,

    /// Number of bytes to dump
    #[arg(short, long)]
    #[arg(default_value_t = 16)]
    pub count: usize,

    /// How to display the start of each row
    #[arg(short, long, name = "TYPE")]
    #[arg(default_value_t = HexdumpLabels::None)]
    pub labels: HexdumpLabels,

    /// Treat the value as an offset into the ELF file
    #[arg(long)]
    pub offset: bool,

    /// Defaults to an address
    #[arg(value_parser = parse_u64_expr)]
    pub value: u64,
}

#[derive(Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum HexdumpLabels {
    /// Show nothing at the start of lines
    None,

    /// Show the address for the first byte on each line
    Addr,

    /// Show the offset from zero for the first byte on each line
    Zero,
}

// TODO add a --limit option to truncate? or just --truncate?
#[derive(Args)]
pub struct StringsArgs {
    /// Section index used to dump just one table
    #[arg(short, long)]
    pub index: Option<usize>,

    /// Max number of results to report for each table, 0 for unlimited
    #[arg(short, long, default_value_t = 10)]
    pub max_results: usize,
}

impl fmt::Display for HexdumpLabels {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HexdumpLabels::None => fmt.write_str("none")?,
            HexdumpLabels::Addr => fmt.write_str("addr")?,
            HexdumpLabels::Zero => fmt.write_str("zero")?,
        }
        Ok(())
    }
}

// TODO this should parse at least simple expressions
fn parse_u64_expr(s: &str) -> Result<u64, String> {
    if s.starts_with("0x") {
        let t = s.trim_start_matches("0x");
        u64::from_str_radix(t, 16).map_err(|_| format!("`{s}` isn't a hex or decimal number"))
    } else {
        s.parse()
            .map_err(|_| format!("`{s}` isn't a hex or decimal number"))
    }
}

// use the open crate to launch off-line docs?
//    maybe a --doc option?
//    would this also be useful for visualization?
//       might be especially good for generated graphical representations
//       maybe memory? or tree data structures, eg pointer refs, or recursive object dumps
