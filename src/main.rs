mod commands;
mod debug;
mod elf;
mod repl;
mod utils;
use crate::elf::ElfFiles;
use crate::repl::ElfAction;
use clap::Parser;
use clap_repl::ClapEditor;
use clap_repl::reedline::{
    DefaultPrompt, FileBackedHistory, Highlighter, Prompt, PromptEditMode, PromptHistorySearch,
    StyledText,
};
use nu_ansi_term::{Color, Style};
use repl::InfoAction;
use repl::Repl;
use std::borrow::Cow;
use std::error::Error;
use std::path::PathBuf;
use std::{io, process};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// paths to a core and/or exe file
    paths: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    utils::generate_style_file();

    let cli = Cli::parse();
    if cli.paths.is_empty() || cli.paths.len() > 2 {
        return Err("expected a path to a core and/or exe file".into());
    }
    let files = ElfFiles::new(cli.paths)?;

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
        Bt => commands::backtrace(io::stdout(), &files),
        Elf(info) => match info.action {
            ElfAction::Header(args) => commands::info_header(io::stdout(), &files, &args),
            ElfAction::Line(args) => commands::info_debug(&files, &args),
            ElfAction::Loads(args) => commands::info_loads(io::stdout(), &files, &args),
            ElfAction::Notes(args) => commands::info_notes(io::stdout(), &files, &args),
            ElfAction::Relocations(args) => commands::info_relocations(io::stdout(), &files, &args),
            ElfAction::Sections(args) => commands::info_sections(io::stdout(), &files, &args),
            ElfAction::Segments(args) => commands::info_segments(io::stdout(), &files, &args),
            ElfAction::Strings(args) => commands::info_strings(io::stdout(), &files, &args),
            ElfAction::Symbols(args) => commands::info_symbols(io::stdout(), &files, &args),
        },
        Find(args) => commands::find(io::stdout(), &files, &args),
        Info(info) => match info.action {
            InfoAction::Line(args) => commands::info_line(&files, &args),
            InfoAction::Mapped(args) => commands::info_mapped(&files, &args),
            InfoAction::Process(args) => commands::info_process(&files, &args),
            InfoAction::Registers(args) => commands::info_registers(&files, &args),
            InfoAction::Signals(args) => commands::info_signals(&files, &args),
        },
        Hexdump(args) => commands::hexdump(io::stdout(), &files, &args),
        Quit => process::exit(0),
    });
    Ok(())
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
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("udb")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        self.default.render_prompt_multiline_indicator()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
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
