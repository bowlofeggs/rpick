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

use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;

use tempfile::NamedTempFile;


const CATEGORY_NOT_FOUND_CONFIG: &str = "
---
test:
  model: gaussian
  choices:
    - option 1
    - option 2
    - option 3
";


// The user should get a useful error message if the requested category does not exist.
#[test]
fn category_not_found() {
    let expected_output =
        "Category does_not_exist not found in config.\n";
    let config_contents = test_rpick_with_config(
        CATEGORY_NOT_FOUND_CONFIG, &mut vec!["does_not_exist"], expected_output);

    // Since the category didn't exist, rpick should not have changed the file.
    assert_eq!(config_contents, CATEGORY_NOT_FOUND_CONFIG);
}


// Assert correct behavior when the config file is not found.
#[test]
fn config_not_found() {
    let expected_output =
        "Error reading config file at /does/not/exist: No such file or directory (os error 2)\n";
    test_rpick(&["-c", "/does/not/exist", "test"], expected_output);
}


// Test rpick with the given inputs and expected outputs.
fn test_rpick_with_config(config: &str, args: &mut Vec<&str>, expected_output: &str) -> String {
    let mut args = args.clone();
    let mut config_f = NamedTempFile::new().expect("Failed to open temp file");
    write!(config_f, "{}", config).expect("Could not write config");
    config_f.as_file_mut().sync_all().unwrap();
    args.append(&mut vec!["-c", config_f.path().to_str().expect("t")]);

    test_rpick(&args, expected_output);

    let mut config_contents = String::new();
    config_f.seek(SeekFrom::Start(0)).expect("Could not seek file");
    config_f.read_to_string(&mut config_contents).expect("Could not read config");
    config_contents
}


fn test_rpick(args: &[&str], expected_output: &str) {
    let mut rpick = Command::new("target/debug/rpick");
    rpick.args(args);

    let proc = rpick.output().expect("Failed to spawn rpick");

    assert_eq!(proc.status.success(), false);
    assert_eq!(std::str::from_utf8(&proc.stdout).unwrap(), expected_output);
}
