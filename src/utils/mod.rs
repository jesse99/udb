pub mod key_map;
pub mod styles;

pub use key_map::*;
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

/// Remove escape sequences from the string (e.g. for colors).
#[cfg(test)]
pub fn strip_escapes(s: &str) -> String {
    // The other way to do this is to change styles.rs to not emit escape sequences for
    // unit tests (and maybe also if some sort of --no-color flag is used). That worked
    // pretty well but even with Style::empty() the tabled crate will add escape sequences
    // to the end of lines to reset all modes.
    let mut result = String::with_capacity(s.len());
    let mut escaping = false;

    // Note that escape sequences can be fairly gnarly, e.g. for RGB colors.
    // See https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
    // There are also escape sequences for things besides text styling but they shouldn't
    // come into play here.
    for c in s.chars() {
        if c == '\x1b' {
            escaping = true;
        } else if escaping {
            if c == 'm' {
                escaping = false;
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
macro_rules! debug_results {
    ($v:ident, $f:ident) => {
        let paths = vec![
            std::path::PathBuf::from("cores/shopping-debug/app-debug"),
            std::path::PathBuf::from("cores/shopping-debug/app-debug.core"),
        ];
        let files = ElfFiles::new(paths).unwrap();
        $f(&mut $v, &files);
    };
    ($v:ident, $f:ident, $a:expr) => {
        let paths = vec![
            std::path::PathBuf::from("cores/shopping-debug/app-debug"),
            std::path::PathBuf::from("cores/shopping-debug/app-debug.core"),
        ];
        let files = ElfFiles::new(paths).unwrap();
        $f(&mut $v, &files, $a);
    };
}
#[cfg(test)]
pub(crate) use debug_results;

#[cfg(test)]
macro_rules! release_results {
    ($v:ident, $f:ident) => {
        let paths = vec![
            std::path::PathBuf::from("cores/shopping-release/app-release"),
            std::path::PathBuf::from("cores/shopping-release/app-release.core"),
        ];
        let files = ElfFiles::new(paths).unwrap();
        $f(&mut $v, &files);
    };
    ($v:ident, $f:ident, $a:expr) => {
        let paths = vec![
            std::path::PathBuf::from("cores/shopping-release/app-release"),
            std::path::PathBuf::from("cores/shopping-release/app-release.core"),
        ];
        let files = ElfFiles::new(paths).unwrap();
        $f(&mut $v, &files, $a);
    };
}
#[cfg(test)]
pub(crate) use release_results;

// macro so insta crate uses a sensible name for the snapshot file
#[cfg(test)]
macro_rules! do_test {
    ($f:ident, debug_only) => {
        // TODO for commands with args will need a variant of this that takes an arg
        let mut v: Vec<u8> = Vec::new();
        debug_results!(v, $f);

        let s = String::from_utf8(v).unwrap();
        let s = crate::utils::strip_escapes(&s);
        insta::assert_snapshot!(s);
    };
    ($f:ident) => {
        let mut v: Vec<u8> = Vec::new();
        debug_results!(v, $f);
        writeln!(&mut v).unwrap();
        release_results!(v, $f);

        let s = String::from_utf8(v).unwrap();
        let s = crate::utils::strip_escapes(&s);
        insta::assert_snapshot!(s);
    };
    ($f:ident, $a:expr) => {
        let mut v: Vec<u8> = Vec::new();
        debug_results!(v, $f, $a);
        writeln!(&mut v).unwrap();
        release_results!(v, $f, $a);

        let s = String::from_utf8(v).unwrap();
        let s = crate::utils::strip_escapes(&s);
        insta::assert_snapshot!(s);
    };
}
#[cfg(test)]
pub(crate) use do_test;
