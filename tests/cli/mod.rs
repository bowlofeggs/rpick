/*
 * Copyright © 2020-2021 Randy Barlow
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
/// This module tests the CLI by running it as a subprocess and inspecting its outputs and
/// resulting config file. This file includes tests from submodules, and also defines a few utility
/// functions that they all use.
use std::io::{Read, Seek, SeekFrom, Write};

use assert_cmd::Command;
use regex::Regex;
use tempfile::NamedTempFile;

mod error_handling;
mod even;
mod gaussian;
mod inventory;
mod lottery;
mod lru;
mod weighted;

// Return which item rpick chose in the given stdout.
//
// # Arguments
//
// * `stdout` - The output from an rpick run.
//
// # Returns
//
// The item that rpick chose.
fn get_pick(stdout: &str) -> String {
    let re = Regex::new(r"Choice is (?P<pick>.*)\.").unwrap();
    let captures = re.captures(&stdout).unwrap();
    captures.name("pick").unwrap().as_str().to_string()
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
fn test_rpick_with_config(
    config: &str,
    args: &mut Vec<&str>,
    stdin: &str,
    expected_success: bool,
) -> (String, String) {
    let mut args = args.clone();
    let mut config_f = NamedTempFile::new().expect("Failed to open temp file");
    write!(config_f, "{}", config).expect("Could not write config");
    config_f.as_file_mut().sync_all().unwrap();
    args.append(&mut vec!["-c", config_f.path().to_str().expect("t")]);

    let stdout = test_rpick(&args, stdin, expected_success);

    let mut config_contents = String::new();
    config_f
        .seek(SeekFrom::Start(0))
        .expect("Could not seek file");
    config_f
        .read_to_string(&mut config_contents)
        .expect("Could not read config");
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
    let mut rpick = Command::cargo_bin("rpick").unwrap();

    let mut assert = rpick.args(args).write_stdin(stdin).assert();

    if expected_success {
        assert = assert.success();
    } else {
        assert = assert.failure();
    }

    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}
