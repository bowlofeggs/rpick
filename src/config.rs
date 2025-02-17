/* Copyright © 2019-2023, 2025 Randy Barlow
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3 of the License.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.*/
//! # Configuration
//!
//! This module defines the rpick configuration.
//!
//! The configuration defines the pick categories, their algorithms, and their choices.
use std::{
    collections::BTreeMap,
    error,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

/// Return the user's config as a BTreeMap.
///
/// # Arguments
///
/// * `config_file_path` - A filesystem path to a YAML file that should be read.
///
/// # Returns
///
/// Returns a mapping of YAML to [`ConfigCategory`]'s, or an Error.
pub fn read_config(
    config_file_path: &Path,
) -> Result<BTreeMap<String, ConfigCategory>, Box<dyn error::Error>> {
    let f = File::open(config_file_path)?;
    let reader = BufReader::new(f);

    let config: BTreeMap<String, ConfigCategory> = serde_yaml::from_reader(reader)?;
    Ok(config)
}

/// Save the data from the given BTreeMap to the user's config file.
///
/// # Arguments
///
/// * `config_file_path` - A filesystem path that the config should be written to.
/// * `config` - The config that should be serialized as YAML.
pub fn write_config(
    config_file_path: &Path,
    config: BTreeMap<String, ConfigCategory>,
) -> Result<(), Box<dyn error::Error>> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_file_path)?;
    let yaml = serde_yaml::to_string(&config).unwrap();

    f.write_all(&yaml.into_bytes())?;
    Ok(())
}

/// A category of items that can be chosen from.
///
/// Each variant of this Enum maps to one of the supported algorithms.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "model")]
pub enum ConfigCategory {
    /// The Even variant picks from its choices with even distribution.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Even { choices: Vec<String> },
    /// The Gaussian variant uses a
    /// [Gaussian distribution](https://en.wikipedia.org/wiki/Normal_distribution) to prefer choices
    /// near the beginning of the list of choices over those at the end. Once a choice has been
    /// accepted, it is moved to the end of the list.
    ///
    /// # Attributes
    ///
    /// * `stddev_scaling_factor` - This is used to derive the standard deviation; the standard
    ///   deviation is the length of the list of choices, divided by this scaling factor.
    /// * `choices` - The list of choices to pick from.
    Gaussian {
        #[serde(default = "default_stddev_scaling_factor")]
        stddev_scaling_factor: f64,
        choices: Vec<String>,
    },
    /// The Inventory variant uses a weighted distribution to pick items, with each items chances
    /// being tied to how many tickets it has. When a choice is accepted, that choice's ticket
    /// count is reduced by 1.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Inventory { choices: Vec<InventoryChoice> },
    /// The Lru variant picks the Least Recently Used item from the list of choices. The least
    /// recently used choice is found at the beginning of the list. Once a choice has been
    /// accepted, it is moved to the end of the list.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    #[serde(rename = "lru")]
    Lru { choices: Vec<String> },
    /// The Lottery variant uses a weighted distribution to pick items, with each items chances
    /// being tied to how many tickets it has. When a choice is accepted, that choice's ticket
    /// count is set to 0, and every choice not chosen receives its weight in additional tickets.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Lottery { choices: Vec<LotteryChoice> },
    /// The Weighted variant is a simple weighted distribution.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Weighted { choices: Vec<WeightedChoice> },
}

/// Represents an individual choice for the inventory model.
///
/// # Attributes
///
/// * `name` - The name of the choice.
/// * `tickets` - The current number of tickets the choice has.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InventoryChoice {
    pub name: String,
    #[serde(default = "default_weight")]
    pub tickets: u64,
}

/// Represents an individual choice for the lottery model.
///
/// # Attributes
///
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LotteryChoice {
    /// The name of the choice
    pub name: String,

    /// How many tickets the choice should be reset to when it is chosen.
    #[serde(default = "default_reset")]
    pub reset: u64,

    /// The current number of tickets the choice has.
    #[serde(default = "default_weight")]
    pub tickets: u64,

    /// The number of tickets that will be added to `tickets` each time this choice is not picked.
    #[serde(default = "default_weight")]
    pub weight: u64,
}

/// Represents an individual choice for the weighted model.
///
/// # Attributes
///
/// * `name` - The name of the choice
/// * `weight` - How much chance this choice has of being chosen, relative to the other choices.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WeightedChoice {
    pub name: String,
    #[serde(default = "default_weight")]
    pub weight: u64,
}

/// Define the default for the stddev_scaling_factor setting as 3.0.
fn default_stddev_scaling_factor() -> f64 {
    3.0
}

/// Reset to 0 by default.
fn default_reset() -> u64 {
    0
}

/// Define the default for the weight setting as 1.
fn default_weight() -> u64 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert!((default_stddev_scaling_factor() - 3.0).abs() < 0.000_001);
        assert_eq!(default_weight(), 1);
        assert_eq!(default_reset(), 0);
    }
}
