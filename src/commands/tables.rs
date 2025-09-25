//! Helpers for building tables using the tabled crate.
use crate::utils::Styling;
use crate::utils::uwriteln;
use std::io::Write;
use tabled::{
    builder::Builder,
    settings::{Alignment, Padding, Style, object::Columns},
};

struct TableCol {
    header: String,
    align: Alignment,
    help: String,
    fields: Vec<String>,
}

/// General table. They look like this:
/// type  offset             vaddr  paddr  file size  memory size  flags   if titles
/// ----  ------             -----  -----  ---------  -----------  -----
/// Note     580                 0      0        d6c            0    ---
/// Load    2000      55957a492000      0       1000         1000    --r
/// Load    3000      55957a493000      0          0         1000    x-r
///
/// type: the segment type                                                 if explain
/// offset: the offset into the ELF file at which the segment appears
/// ...
pub struct TableBuilder {
    cols: Vec<TableCol>,
}

impl TableBuilder {
    pub fn new() -> TableBuilder {
        TableBuilder { cols: Vec::new() }
    }

    /// Left aligned column
    pub fn add_col_l(&mut self, header: &str, help: &str) {
        debug_assert!(!self.has_col(header));
        let col = TableCol {
            header: header.to_string(),
            align: Alignment::left(),
            help: help.to_string(),
            fields: Vec::new(),
        };
        self.cols.push(col);
    }

    /// Right aligned column
    pub fn add_col_r(&mut self, header: &str, help: &str) {
        debug_assert!(!self.has_col(header));
        let col = TableCol {
            header: header.to_string(),
            align: Alignment::right(),
            help: help.to_string(),
            fields: Vec::new(),
        };
        self.cols.push(col);
    }

    /// Typically add_field! is used instead.
    pub fn add_str_field(&mut self, header: &str, value: String) {
        let col = self.find_col(header);
        if value.is_empty() {
            // For some reason empty fields screw up tabled formatting.
            col.fields.push(" ".table_field().to_string());
        } else {
            col.fields.push(value);
        }
    }

    pub fn writeln(&self, mut out: impl Write, titles: bool, explain: bool) {
        uwriteln!(out, "{}", self.table_str(titles));

        if explain {
            uwriteln!(out);
            uwriteln!(out, "{}", self.explain_str());
        }
    }

    // We need to preserve add_col ordering so we can't use a HashMap
    // but O(n) should be fine for tables.
    fn has_col(&self, header: &str) -> bool {
        self.cols.iter().any(|c| c.header == header)
    }

    fn find_col(&mut self, header: &str) -> &mut TableCol {
        self.cols.iter_mut().find(|c| c.header == header).unwrap() // programmer error to not have a col
    }

    fn table_str(&self, titles: bool) -> String {
        let height = self.cols[0].fields.len();
        let mut builder = Builder::with_capacity(height + 2, self.cols.len());
        if titles {
            let names: Vec<String> = self.cols.iter().map(|c| c.header.to_string()).collect();
            let dashes: Vec<String> = names.iter().map(|s| "-".repeat(s.len())).collect();

            let header: Vec<String> = names
                .into_iter()
                .map(|s| s.table_header().to_string())
                .collect();
            let dashes: Vec<String> = dashes
                .into_iter()
                .map(|s| s.table_sep().to_string())
                .collect();
            builder.push_record(&header);
            builder.push_record(&dashes);
        }
        for i in 0..height {
            let row: Vec<String> = self.cols.iter().map(|c| c.fields[i].clone()).collect();
            builder.push_record(&row);
        }

        let mut table = builder.build();
        for (i, col) in self.cols.iter().enumerate() {
            table.modify(Columns::one(i), col.align);
        }
        table.modify(Columns::first(), Padding::new(0, 1, 0, 0));
        table.with(Style::empty());

        table.to_string()
    }

    fn explain_str(&self) -> String {
        let explains: Vec<String> = self
            .cols
            .iter()
            .map(|c| {
                format!(
                    "{}: {}",
                    c.header.clone().explain_title(),
                    c.help.clone().explain_text()
                )
            })
            .collect();
        explains.join("\n")
    }
}

macro_rules! add_field {
    ($builder:ident, $header:literal, $value:expr) => {
        let s = format!("{}", $value);
        let s = s.table_field().to_string();
        $builder.add_str_field($header, s);
    };
    ($builder:ident, $header:literal, $format:literal, $value:expr) => {
        let s = format!($format, $value);
        let s = s.table_field().to_string();
        $builder.add_str_field($header, s);
    };
}
pub(crate) use add_field;

struct SimpleRow {
    name: String,
    value: String,
    help: String,
}

/// Table with just name and value columns. They look like this:
/// little endian        true                    these have no titles
/// 64-bit               true       
///
/// little endian: blah blah                     if explain
/// 64-bit: pointers are eight bytes
pub struct SimpleTableBuilder {
    rows: Vec<SimpleRow>,
}

impl SimpleTableBuilder {
    pub fn new() -> SimpleTableBuilder {
        SimpleTableBuilder { rows: Vec::new() }
    }

    /// Typically add_simple! is used instead.
    pub fn add_str_row(&mut self, name: &str, value: String, help: &str) {
        let row = SimpleRow {
            name: name.to_string(),
            value,
            help: help.to_string(),
        };
        self.rows.push(row);
    }

    pub fn writeln(&self, mut out: impl Write, explain: bool) {
        uwriteln!(out, "{}", self.table_str());

        if explain {
            uwriteln!(out);
            uwriteln!(out, "{}", self.explain_str());
        }
    }

    fn table_str(&self) -> String {
        let height = self.rows.len();
        let mut builder = Builder::with_capacity(height + 2, 2);
        for row in self.rows.iter() {
            let row = vec![row.name.clone(), row.value.clone()];
            builder.push_record(&row);
        }

        let mut table = builder.build();
        table.modify(Columns::one(0), Alignment::left());
        table.modify(Columns::one(1), Alignment::left());
        table.modify(Columns::first(), Padding::new(0, 1, 0, 0));
        table.with(Style::empty());

        table.to_string()
    }

    fn explain_str(&self) -> String {
        let explains: Vec<String> = self
            .rows
            .iter()
            .map(|r| {
                format!(
                    "{}: {}",
                    r.name.clone().explain_title(),
                    r.help.clone().explain_text()
                )
            })
            .collect();
        explains.join("\n")
    }
}

macro_rules! add_simple {
    ($builder:ident, $name:literal, $value:expr, $help:expr) => {
        let s = format!("{}", $value);
        let s = s.table_field().to_string();
        $builder.add_str_row($name, s, $help);
    };
    ($builder:ident, $name:literal, $format:literal, $value:expr, $help:expr) => {
        let s = format!($format, $value);
        let s = s.table_field().to_string();
        $builder.add_str_row($name, s, $help);
    };
}
pub(crate) use add_simple;
