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
//! # The UI Trait
//!
//! The UI Trait defines an interface for bridging human interactions with the rpick crate.

#[cfg(test)]
use mockall::automock;

/// An individual cell within rpick's chance tables.
///
/// Each of the variants expresses its contained type, and should be fairly obvious.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Cell<'a> {
    Boolean(bool),
    Text(&'a str),
    Integer(i64),
    Float(f64),
    Unsigned(u64),
}

impl<'a> From<f64> for Cell<'_> {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl<'a> From<&'a str> for Cell<'a> {
    fn from(s: &'a str) -> Self {
        Self::Text(s)
    }
}

impl<'a> From<u64> for Cell<'_> {
    fn from(u: u64) -> Self {
        Self::Unsigned(u)
    }
}

impl<'a> From<&Cell<'_>> for String {
    fn from(c: &Cell) -> String {
        match c {
            Cell::Boolean(value) => value.to_string(),
            Cell::Text(value) => value.to_string(),
            Cell::Integer(value) => value.to_string(),
            Cell::Float(value) => value.to_string(),
            Cell::Unsigned(value) => value.to_string(),
        }
    }
}

/// Represents a row in the [`Table`] struct.
#[derive(Debug, PartialEq)]
pub struct Row<'a> {
    /// The row's individual cells.
    pub cells: Vec<Cell<'a>>,
    /// Whether this row was chosen in a rpick.
    pub chosen: bool,
}

/// rpick uses this to send a chance table to the user.
#[derive(Debug, PartialEq)]
pub struct Table<'a> {
    /// The Table's footer.
    pub footer: Vec<Cell<'a>>,
    /// The Table's header.
    pub header: Vec<Cell<'a>>,
    /// The Table's rows.
    pub rows: Vec<Row<'a>>,
}

/// A struct implementing this trait must be passed to the rpick engine.
///
/// This is how rpick interacts with users.
#[cfg_attr(test, automock)]
pub trait UI {
    /// If this method returns `true`, [`UI::display_table`] will be called by the engine.
    ///
    /// This is a small optimization - generating tables that the UI isn't going to show to the
    /// user or otherwise use is a waste of compute time. If the table isn't going to get used,
    /// this method should return `false`.
    fn call_display_table(&self) -> bool;

    /// Display the given table to the user.
    fn display_table<'a>(&self, table: &Table<'a>);

    /// Display the given message to the user.
    fn info(&self, message: &str);

    /// Prompt the user if they wish to accept the given choice.
    ///
    /// Return `true` if the user accepts the choice.
    fn prompt_choice(&self, choice: &str) -> bool;
}
