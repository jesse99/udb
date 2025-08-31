pub mod styles;

pub use styles::*;

use std::error::Error;

pub fn require(predicate: bool, err: &str) -> Result<(), Box<dyn Error>> {
    if predicate { Ok(()) } else { Err(err.into()) }
}

pub fn warn(mesg: &str) {
    eprintln!("{}", mesg.warn());
}

pub fn explain(title: &str, text: &str) {
    println!("{}: {}", title.explain_title(), text.explain_text());
}

pub fn align_to_word(n: u32) -> u32 {
    (n + 3) & !3
}
