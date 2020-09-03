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

const GAUSSIAN_CONFIG: &str = "
---
gaussian:
  model: gaussian
  choices:
    - option 1
    - option 2
    - option 3
";

const INVENTORY_CONFIG: &str = "
---
inventory:
  model: inventory
  choices:
    - name: option 1
      tickets: 1
    - name: option 2
      tickets: 1
    - name: option 3
      tickets: 1
    - name: option 4
      # This one should never get picked
      tickets: 0
";

const LOTTERY_CONFIG: &str = "
---
lottery:
  model: lottery
  choices:
    - name: option 1
      weight: 1
      tickets: 1
    - name: option 2
      weight: 2
      tickets: 1
    - name: option 3
      weight: 3
      tickets: 1
    - name: option 4
      weight: 4
      # This one should never get picked
      tickets: 0
";

const LRU_CONFIG: &str = "
---
lru:
  model: lru
  choices:
    - option 1
    - option 2
    - option 3
";

const WEIGHTED_CONFIG: &str = "
---
weighted:
  model: weighted
  choices:
    - name: option 1
      weight: 1
    - name: option 2
      weight: 2
    - name: option 3
      weight: 3
    - name: option 4
      # This one should never get picked
      weight: 0
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
    let pick = get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
}


// Assert correct behavior with an gaussian model config
#[test]
fn gaussian_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        GAUSSIAN_CONFIG, &mut vec!["gaussian"], "y\n", true);

    // Assert that the chosen item was a member of the config
    let expected_values: HashSet<&'static str> =
        ["option 1", "option 2", "option 3"].iter().cloned().collect();
    let pick = get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
    // Assert that the gaussian model moves the picked item into last place
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&GAUSSIAN_CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Gaussian{choices, stddev_scaling_factor: _}
            = &mut expected_config.get_mut("gaussian").unwrap() {
        let index = choices.iter().position(|x| x == pick.as_str()).unwrap();
        choices.remove(index);
        choices.push(pick);
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}


// Assert correct behavior with an inventory model config
#[test]
fn inventory_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        INVENTORY_CONFIG, &mut vec!["inventory"], "y\n", true);

    // Assert that the chosen item was a member of the config. Note that "option 4" is not listed
    // here, though it is in the config, since it has 0 tickets and should never be chosen.
    let expected_values: HashSet<&'static str> =
        ["option 1", "option 2", "option 3"].iter().cloned().collect();
    let pick = get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
    // Assert that the inventory model reduces the tickets on the picked item
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&INVENTORY_CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Inventory{choices}
            = &mut expected_config.get_mut("inventory").unwrap() {
        let index = choices.iter().position(|x| x.name == pick).unwrap();
        choices[index].tickets = 0;
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}


// Assert correct behavior with an lottery model config
#[test]
fn lottery_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        LOTTERY_CONFIG, &mut vec!["lottery"], "y\n", true);

    // Assert that the chosen item was a member of the config. Note that "option 4" is not listed
    // here, though it is in the config, since it has 0 tickets and should never be chosen.
    let expected_values: HashSet<&'static str> =
        ["option 1", "option 2", "option 3"].iter().cloned().collect();
    let pick = get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
    // Assert that the lottery model removes the tickets on the picked item, and gives more tickets
    // to the ones that weren't picked.
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&LOTTERY_CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Lottery{choices}
            = &mut expected_config.get_mut("lottery").unwrap() {
        for choice in choices.iter_mut() {
            if choice.name == pick {
                choice.tickets = 0;
            } else {
                choice.tickets += choice.weight;
            }
        }
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}


// Assert correct behavior with an lru model config
#[test]
fn lru_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        LRU_CONFIG, &mut vec!["lru"], "y\n", true);

    // Assert that the chosen item was a member of the config
    assert_eq!(get_pick(&stdout), "option 1");
    // Assert that the lru model moves the picked item into last place
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&LRU_CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::LRU{choices}
            = &mut expected_config.get_mut("lru").unwrap() {
        let pick = choices.remove(0);
        choices.push(pick);
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}


// Assert correct behavior with an weighted distribution model config
#[test]
fn weighted_pick() {
    let (stdout, config_contents) = test_rpick_with_config(
        WEIGHTED_CONFIG, &mut vec!["weighted"], "y\n", true);

    let expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&WEIGHTED_CONFIG).expect("Could not parse yaml");
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    // The weighted config does not modify the config file
    assert_eq!(parsed_config, expected_config);
    // Assert that the chosen item was a member of the config. Note that "option 4" does not appear
    // here since it has a weight of 0, meaning it should never get chosen.
    let expected_values: HashSet<&'static str> =
        ["option 1", "option 2", "option 3"].iter().cloned().collect();
    let pick = get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
}


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
