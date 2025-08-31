mod commands;
mod debug;
mod elf;
mod repl;
mod utils;
use clap::Parser;
use clap_repl::ClapEditor;
use clap_repl::reedline::{
    DefaultPrompt, FileBackedHistory, Highlighter, Prompt, PromptEditMode, PromptHistorySearch,
    StyledText,
};
use elf::ElfFile;
use nu_ansi_term::{Color, Style};
use repl::InfoAction;
use repl::Repl;
use std::borrow::Cow;
use std::path::PathBuf;
use std::process;

use crate::utils::warn;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// path to core file
    core: PathBuf,
}

/// A simple, example highlighter that shows how to highlight keywords
pub struct MyHighlighter {
    color: Color,
}

impl Highlighter for MyHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled_text = StyledText::new();

        styled_text.push((Style::new().fg(self.color), line.to_string()));

        styled_text
    }
}

impl MyHighlighter {
    pub fn new() -> MyHighlighter {
        MyHighlighter { color: Color::Blue }
    }
}

impl Default for MyHighlighter {
    fn default() -> Self {
        MyHighlighter::new()
    }
}

pub struct MyPrompt {
    color: clap_repl::reedline::Color,
    default: DefaultPrompt,
}

impl Prompt for MyPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed("udb")
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, prompt_mode: PromptEditMode) -> Cow<str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        self.default.render_prompt_multiline_indicator()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        self.default
            .render_prompt_history_search_indicator(history_search)
    }

    // the text that appears in the prompt
    fn get_prompt_color(&self) -> clap_repl::reedline::Color {
        self.color
    }

    // the symbol that typically appears in the prompt, e.g. '>'
    fn get_indicator_color(&self) -> clap_repl::reedline::Color {
        clap_repl::reedline::Color::Black // TODO doesn't seem to work
    }
}

impl MyPrompt {
    fn new() -> MyPrompt {
        MyPrompt {
            color: clap_repl::reedline::Color::DarkBlue,
            default: DefaultPrompt::default(),
        }
    }
}

fn load_core(path: PathBuf) -> ElfFile {
    match ElfFile::new(path.clone()) {
        Ok(core) => core,
        Err(e) => {
            warn(&format!("Couldn't load {}: {e}", path.display()));
            std::process::exit(1);
        }
    }
}
// TODO
// add an `info sections` command
// can we get function names from the function pointers?
// can we get the function parameters?
// rename Core? maybe to Elf?
// backup
// verify that pem core still works
// need a test for not a core file
// add a todo.rtf file
// probably want to uninstall mactex, see https://www.tug.org/mactex/uninstalling.html
//    maybe try to install the basic version
// add some interactive commands
//    should be able to run a command from bash
//    maybe allow them to be chained somehow
// add tests for command output?
//    have to be careful with stuff like addresses
//    or is it better to have tests for the backend data?
//       that's probably more stable
//       tho it means the formatted output isnt tested...
fn main() {
    utils::generate_style_file();

    let cli = Cli::parse();
    let path = cli.core;
    let core = load_core(path);

    // left prompt                    before what the user types
    // highlighter                    this is for what the user types
    // with_visual_selection_style    this is for the selection
    let prompt = MyPrompt::new(); // TODO should be able to configure stuff like colors and history size
    let rl = ClapEditor::<Repl>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            reed.with_highlighter(Box::new(MyHighlighter::new()))
                .with_history(Box::new(
                    FileBackedHistory::with_file(10000, "/tmp/udb-history".into()).unwrap(),
                ))
        })
        .build();

    use repl::MainCommand::*;
    rl.repl(|repl: Repl| match repl.command {
        Bt => commands::backtrace(&core),
        Find(args) => commands::find(&core, &args),
        Info(info) => match info.action {
            InfoAction::Header(args) => commands::info_header(&core, &args),
            InfoAction::Loads(args) => commands::info_loads(&core, &args),
            InfoAction::Mapped(args) => commands::info_mapped(&core, &args),
            InfoAction::Process(args) => commands::info_process(&core, &args),
            InfoAction::Registers(args) => commands::info_registers(&core, &args),
            InfoAction::Sections(args) => commands::info_sections(&core, &args),
            InfoAction::Segments(args) => commands::info_segments(&core, &args),
            InfoAction::Signals(args) => commands::info_signals(&core, &args),
            InfoAction::Symbols(args) => commands::info_symbols(&core, &args),
        },
        Hexdump(args) => commands::hexdump(&core, &args),
        Quit => process::exit(0),
    });
}
