/* Copyright © 2019-2020, 2025 Randy Barlow
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.*/
//! This is the rpick CLI struct.
//!
//! It is unlikely that you would want to use this crate in your code. It exists as a separate
//! crate so that the rpick `build.rs` can use the CLI struct to generate a man page.

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about, version)]
pub struct CliArgs {
    /// The category you wish to pick from.
    pub category: String,

    /// A path to the config file you wish to use.
    #[arg(short, long, env = "RPICK_CONFIG")]
    pub config: Option<PathBuf>,

    /// Print more information about the pick.
    #[arg(short, long)]
    pub verbose: bool,
}
