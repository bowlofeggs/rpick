/* Copyright © 2019 Randy Barlow
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.*/
//! # rpick
//!
//! ```rpick``` helps pick items from a list of choices, using a Gaussian distribution.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::{self, BufRead};
use std::io::prelude::*;

use rand::distributions::{Distribution,Normal};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;


const CONFIG_FILE: &str = "rpick.yml";


#[derive(StructOpt)]
struct Cli {
    /// The category you wish to pick from.
    category: String,
}


#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum CategoryType {
    gaussian,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ConfigCategory {
    model: CategoryType,
    choices: Vec<String>,
}


fn main() {
    let args = Cli::from_args();
    let mut config = _read_config();
    let stddev = (config[&args.category].choices.len() as f64) / 3.0;
    let normal = Normal::new(0.0, stddev);
    let mut accept = false;
    let mut index = 0;

    while !accept {
        index = loop {
            let index = normal.sample(&mut rand::thread_rng()).abs() as usize;
            if index < config[&args.category].choices.len() {
                break index;
            }
        };

        print!("Choice is {}. Accept? (Y/n) ", config[&args.category].choices[index]);
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let line1 = stdin.lock().lines().next().unwrap().unwrap();
        if ["", "y", "Y"].contains(&line1.as_str()) {
            accept = true;
        }
    }

    let value = config.get_mut(&args.category).expect("category not found").choices.remove(index);
    config.get_mut(&args.category).expect("category not found").choices.push(value);

    _write_config(config);
}


/// Return the path to the user's config file.
fn _get_config_file_path() -> String {
    let config_dir = dirs::config_dir().expect("Unable to find config dir.");
    let config_file = config_dir.join(CONFIG_FILE);
    return String::from(config_file.to_str().expect("Unable to determine config."));
}


/// Return the user's config as a BTreeMap.
fn _read_config() -> BTreeMap<String, ConfigCategory> {
    let config_file_path = _get_config_file_path();
    let f = File::open(&config_file_path);

    let mut f = match f {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                return BTreeMap::new();
            },
            other_error => panic!("There was a problem opening the file: {:?}", other_error),
        },
    };

    let mut yaml_string = String::new();
    let error_msg = format!("Could not read {}", &config_file_path);
    f.read_to_string(&mut yaml_string).expect(&error_msg);
    let config: BTreeMap<String, ConfigCategory> = serde_yaml::from_str(
        &yaml_string).expect("Unable to parse config file.");
    return config;
}


/// Save the data from the given BTreeMap to the user's config file.
fn _write_config(config: BTreeMap<String, ConfigCategory>) {
    let config_file_path = _get_config_file_path();
    let f = OpenOptions::new().write(true).create(true).truncate(true).open(
        &config_file_path);
    let error_msg = format!("Could not write {}", &config_file_path);
    let mut f = f.expect(&error_msg);
    let yaml = serde_yaml::to_string(&config).unwrap();

    f.write_all(&yaml.into_bytes()).expect("Could not write {}");
}
