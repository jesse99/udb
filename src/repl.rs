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

    /// Search memory for a bit pattern
    Find(FindArgs),

    /// Show various forms of information
    Info(InfoCommand),

    /// Print memory range as hex and ascii
    Hexdump(HexdumpArgs),

    /// Exit udb
    Quit,
}

#[derive(Args)]
pub struct InfoCommand {
    #[clap(subcommand)]
    pub action: InfoAction,
}

#[derive(Subcommand)]
pub enum InfoAction {
    /// Show ELF header
    Header(ExplainArgs),

    /// Show ELF load segments
    Loads(TableArgs),

    /// Show memory mapped files
    Mapped(TableArgs),

    /// Show information the process associated with the core file
    Process(ExplainArgs),

    /// Show general purpose registers
    Registers(RegistersArgs),

    /// Show sections
    Sections(TableArgs),

    /// Show segments
    Segments(TableArgs),

    /// Show information about signals
    Signals(TableArgs),

    /// Show symbols
    Symbols(TableArgs),
}

#[derive(Args)]
pub struct ExplainArgs {
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
pub struct TableArgs {
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

    /// Explain columns, fields, etc.
    #[arg(short, long)]
    pub explain: bool,

    /// Add column headers
    #[arg(short, long)]
    pub titles: bool,
}

#[derive(Args)]
pub struct HexdumpArgs {
    /// Address at which to start dumping
    #[arg(value_parser = parse_u64_expr)]
    pub addr: u64,

    /// Number of bytes to dump
    #[arg(short, long)]
    #[arg(default_value_t = 16)]
    pub count: usize,

    /// How to display offsets for the start of each row
    #[arg(short, long, name = "TYPE")]
    #[arg(default_value_t = HexdumpOffsets::None)]
    pub offsets: HexdumpOffsets,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum HexdumpOffsets {
    /// Don't show offsets
    None,

    /// Show offsets starting at addr
    Addr,

    /// Show offsets starting at zero
    Zero,
}

impl fmt::Display for HexdumpOffsets {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HexdumpOffsets::None => fmt.write_str("none")?,
            HexdumpOffsets::Addr => fmt.write_str("addr")?,
            HexdumpOffsets::Zero => fmt.write_str("zero")?,
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
