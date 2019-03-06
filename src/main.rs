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
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead};
use std::io::prelude::*;
use std::io::BufReader;

use rand::distributions::{Distribution,Normal};
use serde::{Serialize, Deserialize};
use structopt::StructOpt;


const CONFIG_FILE: &str = "rpick.yml";


#[derive(StructOpt)]
struct Cli {
    /// The category you wish to pick from.
    category: String,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum CategoryType {
    #[serde(rename="gaussian")]
    Gaussian,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigCategory {
    #[serde(default = "_default_model")]
    model: CategoryType,
    #[serde(default = "_default_stddev_scaling_factor")]
    stddev_scaling_factor: f64,
    choices: Vec<String>,
}


fn main() {
    let args = Cli::from_args();
    let mut config = _read_config().unwrap();
    let stddev = (config[&args.category].choices.len() as f64) /
        config[&args.category].stddev_scaling_factor;
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


/// Define the default for the model setting as Gaussian.
fn _default_model() -> CategoryType {
    return CategoryType::Gaussian;
}


/// Define the default for the stddev_scaling_factor setting as 3.0.
fn _default_stddev_scaling_factor() -> f64 {
    return 3.0;
}


/// Return the path to the user's config file.
fn _get_config_file_path() -> String {
    let config_dir = dirs::config_dir().expect("Unable to find config dir.");
    let config_file = config_dir.join(CONFIG_FILE);
    return String::from(config_file.to_str().expect("Unable to determine config."));
}


/// Return the user's config as a BTreeMap.
fn _read_config() -> Result<BTreeMap<String, ConfigCategory>, Box<Error>> {
    let config_file_path = _get_config_file_path();
    let f = File::open(&config_file_path)?;
    let reader = BufReader::new(f);

    let config: BTreeMap<String, ConfigCategory> = serde_yaml::from_reader(reader)?;
    return Ok(config);
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
