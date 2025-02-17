/* Copyright © 2021 Randy Barlow
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.*/
//! Define the code that drives the rpick CLI UI.

use std::io::{self, BufRead, Write};

use prettytable::{format, Cell, Row, Table};

use rpick::ui;

/// This implements the Ui trait for the rpick engine.
pub struct Cli {
    /// If true, print out the chance tables.
    verbose: bool,
}

impl Cli {
    /// Construct a new Cli.
    ///
    /// # Arguments
    ///
    /// * `verbose`: If true, the Cli will print out chance tables.
    pub fn new(verbose: bool) -> Self {
        Cli { verbose }
    }

    /// Convert a slice of Cells into a [`prettytable::Row`].
    ///
    /// # Arguments
    ///
    /// * `row`: The slice of Cells to convert.
    /// * `highlight`: If true, this row will get emphasized on terminals that support colors.
    fn convert_row(row: &[ui::Cell], highlight: bool) -> Row {
        let mut r = Row::empty();

        for c in row {
            let mut c = if let ui::Cell::Float(value) = c {
                Cell::new(&format!("{:>6.2}%", value))
            } else {
                Cell::new(&String::from(c))
            };
            if highlight {
                c = c.style_spec("bFy");
            }
            r.add_cell(c);
        }

        r
    }
}

impl ui::Ui for Cli {
    /// Return `self.verbose`.
    fn call_display_table(&self) -> bool {
        self.verbose
    }

    /// Print the given table to the terminal.
    fn display_table(&self, table: &ui::Table) {
        let mut t = Table::new();
        t.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        t.set_titles(Cli::convert_row(&table.header, false));

        for row in &table.rows {
            t.add_row(Cli::convert_row(&row.cells, row.chosen));
        }
        t.add_row(Cli::convert_row(&table.footer, false));

        println!();
        t.printstd();
        println!();
    }

    /// Print the given message to the terminal.
    fn info(&self, message: &str) {
        println!("{}", message);
    }

    /// Ask the user if they accept the given choice and return their answer.
    fn prompt_choice(&self, choice: &str) -> bool {
        print!("Choice is {}. Accept? (Y/n) ", choice);
        io::stdout().flush().unwrap();
        let line = io::stdin().lock().lines().next().unwrap().unwrap();
        if ["", "y", "Y"].contains(&line.as_str()) {
            return true;
        }
        false
    }
}
