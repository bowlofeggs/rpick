/*
 * Copyright © 2020 Randy Barlow
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::process::Command;


// Assert correct behavior when the config file is not found.
#[test]
fn config_not_found() {
    let mut rpick = Command::new("target/debug/rpick");
    rpick.args(vec!["-c", "/does/not/exist", "test"]);

    let proc = rpick.output().expect("Failed to spawn rpick");

    assert_eq!(proc.status.success(), false);
    let expected_output =
        "Error reading config file at /does/not/exist: No such file or directory (os error 2)\n";
    assert_eq!(std::str::from_utf8(&proc.stdout).unwrap(), expected_output);
}
