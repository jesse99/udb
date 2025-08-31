//! Used to color and otherwise style various bits of output using a
//! ~/.udb/styles.tcss file.
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::sync::LazyLock;
use std::{fs, path::PathBuf};
use termio::prelude::*;
use termio::{StyledString, Termio};

/// Create the style file if it is missing.
pub fn generate_style_file() {
    // TODO should merge in missing elements
    if let Some(mut path) = dirs::home_dir() {
        path.push(".udb");
        if make_dir(&path) {
            path.push("styles.tcss");
            default_styles(path);
        }
    } else {
        println!("couldn't find home directory"); // don't use warn() here
    }
}

macro_rules! print_styled {
    ($format:expr, $style:ident) => {
        let s = format!($format).$style();
        print!("{s}");
    };
    ($format:expr, $style:ident, $arg1:expr) => {
        let s = format!($format, $arg1).$style();
        print!("{s}");
    };
    ($format:expr, $style:ident, $arg1:expr, $arg2:expr) => {
        let s = format!($format, $arg1, $arg2).$style();
        print!("{s}");
    };
}
pub(crate) use print_styled;

pub trait Styling {
    fn explain_title(self) -> StyledString;
    fn explain_text(self) -> StyledString;
    fn hex_offset(self) -> StyledString;
    fn hex_hex(self) -> StyledString;
    fn hex_ascii(self) -> StyledString;
    fn table_header(self) -> StyledString;
    fn table_sep(self) -> StyledString;
    fn table_field(self) -> StyledString;
    fn warn(self) -> StyledString;
}

impl Styling for String {
    fn explain_title(self) -> StyledString {
        self.style("explain title", &TCSS)
    }

    fn explain_text(self) -> StyledString {
        self.style("explain text", &TCSS)
    }

    fn hex_offset(self) -> StyledString {
        self.style("hex offset", &TCSS)
    }

    fn hex_hex(self) -> StyledString {
        self.style("hex hex", &TCSS)
    }

    fn hex_ascii(self) -> StyledString {
        self.style("hex ascii", &TCSS)
    }

    fn table_header(self) -> StyledString {
        self.style("table header", &TCSS)
    }

    fn table_sep(self) -> StyledString {
        self.style("table separator", &TCSS)
    }

    fn table_field(self) -> StyledString {
        self.style("table field", &TCSS)
    }

    fn warn(self) -> StyledString {
        self.style("warn", &TCSS)
    }
}

impl Styling for &str {
    fn explain_title(self) -> StyledString {
        self.style("explain title", &TCSS)
    }

    fn explain_text(self) -> StyledString {
        self.style("explain text", &TCSS)
    }

    fn hex_offset(self) -> StyledString {
        self.style("hex offset", &TCSS)
    }

    fn hex_hex(self) -> StyledString {
        self.style("hex hex", &TCSS)
    }

    fn hex_ascii(self) -> StyledString {
        self.style("hex ascii", &TCSS)
    }

    fn table_header(self) -> StyledString {
        self.style("table header", &TCSS)
    }

    fn table_sep(self) -> StyledString {
        self.style("table separator", &TCSS)
    }

    fn table_field(self) -> StyledString {
        self.style("table field", &TCSS)
    }

    fn warn(self) -> StyledString {
        self.style("warn", &TCSS)
    }
}

static TCSS: LazyLock<Termio> = LazyLock::new(|| {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".udb");
        path.push("styles.tcss");
        let os_path = path.into_os_string().into_string().unwrap();
        match Termio::from_file(&os_path) {
            Ok(tcss) => tcss,
            Err(err) => {
                println!("could't parse file at {os_path}: {err}"); // don't use warn() here
                Termio::new()
            }
        }
    } else {
        Termio::new() // we'll have warned about this already
    }
});

fn make_dir(path: &Path) -> bool {
    match fs::create_dir(path) {
        Ok(_) => true,
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => true,
            _ => {
                println!("could't create path for {}: {err}", path.display()); // don't use warn() here
                false
            }
        },
    }
}

fn default_styles(path: PathBuf) {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.clone())
    {
        Ok(mut file) => {
            let defaults = include_str!("default.tcss");
            if let Err(err) = file.write_all(defaults.as_bytes()) {
                println!("error writing defaults to {}: {err}", path.display());
            }
        }
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => (), // user already has a styles file
            _ => println!("error creating {}: {err}", path.display()), // don't use warn() here
        },
    }
}
