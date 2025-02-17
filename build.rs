/* Copyright © 2025 Randy Barlow
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.*/
//! Build rpick's man page.

use std::path::PathBuf;

use clap::{CommandFactory, Parser};

include!("src/command.include");

fn main() -> std::io::Result<()> {
    let out_dir =
        std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?)
            .join("..")
            .join("..")
            .join("..");

    let man = clap_mangen::Man::new(CliArgs::command());
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("rpick.1"), buffer)?;

    Ok(())
}
