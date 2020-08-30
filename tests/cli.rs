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

use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};

use regex::Regex;
use tempfile::NamedTempFile;

use rpick::ConfigCategory;


const CATEGORY_NOT_FOUND_CONFIG: &str = "
---
test:
  model: gaussian
  choices:
    - option 1
    - option 2
    - option 3
";

const EVEN_CONFIG: &str = "
---
even:
  model: even
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

    let (stdout, config_contents) = test_rpick_with_config(
        CATEGORY_NOT_FOUND_CONFIG, &mut vec!["does_not_exist"], "", false);

    assert_eq!(stdout, expected_output);
    // Since the category didn't exist, rpick should not have changed the file.
    assert_eq!(config_contents, CATEGORY_NOT_FOUND_CONFIG);
}


// Assert correct behavior when the config file is not found.
#[test]
fn config_not_found() {
    let expected_output =
        "Error reading config file at /does/not/exist: No such file or directory (os error 2)\n";

    let stdout = test_rpick(&["-c", "/does/not/exist", "test"], "", false);

    assert_eq!(stdout, expected_output);
}


// Assert correct behavior with an even distribution model config
#[test]
fn even_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        EVEN_CONFIG, &mut vec!["even"], "y\n", true);

    let expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&EVEN_CONFIG).expect("Could not parse yaml");
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    // The even config does not modify the config file
    assert_eq!(parsed_config, expected_config);
    // Assert that the chosen item was a member of the config
    let expected_values: HashSet<&'static str> =
        ["option 1", "option 2", "option 3"].iter().cloned().collect();
    let re = Regex::new(r"Choice is (?P<pick>.*)\.").unwrap();
    let captures = re.captures(&stdout).unwrap();
    assert_eq!(expected_values.contains(captures.name("pick").unwrap().as_str()), true);
}


// Run rpick with the given config, arguments, and stdin.
//
// # Arguments
//
// * `config` - The configuration to test rpick with.
// * `args` - A list of command line arguments to pass to rpick.
// * `stdin` - stdin input to rpick, to simulate a user typing.
// * `expected_success` - If true, assert that the exit code is 0, else assert that it is not 0.
//
// # Returns
//
// Return stdout from rpick, and the contents of the config after running, so that tests can perform
// further assertions.
fn test_rpick_with_config(config: &str, args: &mut Vec<&str>, stdin: &str, expected_success: bool)
        -> (String, String) {
    let mut args = args.clone();
    let mut config_f = NamedTempFile::new().expect("Failed to open temp file");
    write!(config_f, "{}", config).expect("Could not write config");
    config_f.as_file_mut().sync_all().unwrap();
    args.append(&mut vec!["-c", config_f.path().to_str().expect("t")]);

    let stdout = test_rpick(&args, stdin, expected_success);

    let mut config_contents = String::new();
    config_f.seek(SeekFrom::Start(0)).expect("Could not seek file");
    config_f.read_to_string(&mut config_contents).expect("Could not read config");
    (stdout, config_contents)
}


// Run rpick with the given arguments and stdin.
//
// # Arguments
//
// * `args` - A list of command line arguments to pass to rpick.
// * `stdin` - stdin input to rpick, to simulate a user typing.
// * `expected_success` - If true, assert that the exit code is 0, else assert that it is not 0.
//
// # Returns
//
// Return stdout from rpick, so that tests can perform further assertions.
fn test_rpick(args: &[&str], stdin: &str, expected_success: bool) -> String {
    let mut rpick = Command::new("target/debug/rpick").args(args)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("Failed to spawn rpick");
    let stdin_pipe = rpick.stdin.as_mut().expect("failed to get stdin");
    stdin_pipe.write_all(stdin.as_bytes()).expect("failed to write to stdin");

    let proc = rpick.wait_with_output().expect("Failed to spawn rpick");

    assert_eq!(proc.status.success(), expected_success);
    String::from_utf8(proc.stdout).unwrap()
}
