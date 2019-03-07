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
//! ```rpick``` helps pick items from a list of choices, using various algorithms.
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead};
use std::io::prelude::*;
use std::io::BufReader;

use rand::distributions::{Distribution,Normal};
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};
use structopt::StructOpt;


const CONFIG_FILE: &str = "rpick.yml";


#[derive(StructOpt)]
struct Cli {
    /// The category you wish to pick from.
    category: String,
}


#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "model")]
enum ConfigCategory {
    Even {
        choices: Vec<String>
    },
    Gaussian {
        #[serde(default = "_default_stddev_scaling_factor")]
        stddev_scaling_factor: f64,
        choices: Vec<String>
    },
    Weighted {
        choices: Vec<WeightedChoice>
    }
}


#[derive(PartialEq, Serialize, Deserialize)]
struct WeightedChoice {
    name: String,
    #[serde(default = "_default_weight")]
    weight: u64,
}


fn main() {
    let args = Cli::from_args();
    let mut config = _read_config().unwrap();
    let category = config.get_mut(&args.category).expect("category not found");
    match category {
        ConfigCategory::Even { choices } => {
            _pick_even(choices);
        }
        ConfigCategory::Gaussian { choices, stddev_scaling_factor } => {
            _pick_gaussian(choices, *stddev_scaling_factor);
            _write_config(config);
        }
        ConfigCategory::Weighted { choices } => {
            _pick_weighted(choices);
        }
    }
}


/// Define the default for the stddev_scaling_factor setting as 3.0.
fn _default_stddev_scaling_factor() -> f64 {
    return 3.0;
}


/// Define the default for the weight setting as 1.
fn _default_weight() -> u64 {
    return 1;
}


/// Return the path to the user's config file.
fn _get_config_file_path() -> String {
    let config_dir = dirs::config_dir().expect("Unable to find config dir.");
    let config_file = config_dir.join(CONFIG_FILE);
    return String::from(config_file.to_str().expect("Unable to determine config."));
}


/// Prompt the user for consent for the given choice, returning a bool true if they accept the
/// choice, or false if they do not.
fn _get_consent(choice: &str) -> bool {
    print!("Choice is {}. Accept? (Y/n) ", choice);
    io::stdout().flush().unwrap();
    let stdin = io::stdin();
    let line1 = stdin.lock().lines().next().unwrap().unwrap();
    if ["", "y", "Y"].contains(&line1.as_str()) {
        return true;
    }
    return false;
}


/// Use an even distribution random model to pick from the given choices.
fn _pick_even(choices: &mut Vec<String>) {
    let choices = choices.iter().map(|x| (x, 1)).collect::<Vec<_>>();

    loop {
        let mut rng = rand::thread_rng();
        let choice = choices.choose_weighted(&mut rng, |item| item.1).unwrap().0;

        if _get_consent(choice) {
            break;
        }
    }
}


/// Run the gaussian model for the given choices and standard deviation scaling factor. When the
/// user accepts a choice, move that choice to end of the choices Vector and return.
fn _pick_gaussian(choices: &mut Vec<String>, stddev_scaling_factor: f64) {
    let stddev = (choices.len() as f64) / stddev_scaling_factor;
    let normal = Normal::new(0.0, stddev);
    let mut index;

    loop {
        index = loop {
            index = normal.sample(&mut rand::thread_rng()).abs() as usize;
            if index < choices.len() {
                break index;
            }
        };

        if _get_consent(&choices[index][..]) {
            break;
        }
    }

    let value = choices.remove(index);
    choices.push(value);
}


/// Run the weighted model for the given choices.
fn _pick_weighted(choices: &mut Vec<WeightedChoice>) {
    let choices = choices.iter().map(|x| (&x.name, x.weight)).collect::<Vec<_>>();

    loop {
        let mut rng = rand::thread_rng();
        let choice = choices.choose_weighted(&mut rng, |item| item.1).unwrap().0;

        if _get_consent(&choice[..]) {
            break;
        }
    }
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
