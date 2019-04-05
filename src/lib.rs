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
//!
use std::collections::BTreeMap;
use std::error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use rand::distributions::{Distribution,Normal};
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};


pub struct Engine<I, O, R> {
    pub input: I,
    pub output: O,
    pub rng: R
}


impl<I, O, R> Engine<I, O, R>
where
    I: BufRead,
    O: Write,
    R: rand::RngCore,
{
    pub fn pick(&mut self, config: &mut BTreeMap<String, ConfigCategory>, category: String)
            -> Result<(), String> {
        let config_category = config.get_mut(&category[..]);
        match config_category {
            Some(category) => {
                match category {
                    ConfigCategory::Even { choices } => {
                        self.pick_even(choices);
                    }
                    ConfigCategory::Gaussian { choices, stddev_scaling_factor } => {
                        self.pick_gaussian(choices, *stddev_scaling_factor);
                    }
                    ConfigCategory::Lottery { choices } => {
                        self.pick_lottery(choices);
                    }
                    ConfigCategory::Weighted { choices } => {
                        self.pick_weighted(choices);
                    }
                }
                Ok(())
            }
            None => {
                Err(format!("Category {} not found in config.", category))
            }
        }
    }

    /// Prompt the user for consent for the given choice, returning a bool true if they accept the
    /// choice, or false if they do not.
    fn get_consent(&mut self, choice: &str) -> bool {
        write!(self.output, "Choice is {}. Accept? (Y/n) ", choice).expect(
            "Could not write to output");
        self.output.flush().unwrap();
        let line1 = self.input.by_ref().lines().next().unwrap().unwrap();
        if ["", "y", "Y"].contains(&line1.as_str()) {
            return true;
        }
        return false;
    }

    /// Use an even distribution random model to pick from the given choices.
    fn pick_even(&mut self, choices: &Vec<String>) {
        let choices = choices.iter().map(|x| (x, 1)).collect::<Vec<_>>();

        loop {
            let choice = choices.choose_weighted(&mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(choice) {
                break;
            }
        }
    }


    /// Run the gaussian model for the given choices and standard deviation scaling factor. When the
    /// user accepts a choice, move that choice to end of the choices Vector and return.
    fn pick_gaussian(&mut self, choices: &mut Vec<String>, stddev_scaling_factor: f64) {
        let stddev = (choices.len() as f64) / stddev_scaling_factor;
        let normal = Normal::new(0.0, stddev);
        let mut index;

        loop {
            index = normal.sample(&mut self.rng).abs() as usize;
            match choices.get(index) {
                Some(value) => {
                    if self.get_consent(&value[..]) {
                        break;
                    }
                },
                None => ()
            }
        }

        let value = choices.remove(index);
        choices.push(value);
    }


    /// Run the lottery model for the given choices.
    fn pick_lottery(&mut self, choices: &mut Vec<LotteryChoice>) {
        let weighted_choices = choices.iter().enumerate().map(
            |x| ((x.0, &x.1.name), x.1.tickets)).collect::<Vec<_>>();

        let index = loop {
            let (index, choice) = weighted_choices.choose_weighted(
                &mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(&choice[..]) {
                break index;
            }
        };

        for choice in choices.iter_mut() {
            choice.tickets += choice.weight;
        }
        choices[index].tickets = 0;
    }


    /// Run the weighted model for the given choices.
    fn pick_weighted(&mut self, choices: &Vec<WeightedChoice>) {
        let choices = choices.iter().map(|x| (&x.name, x.weight)).collect::<Vec<_>>();

        loop {
            let choice = choices.choose_weighted(&mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(&choice[..]) {
                break;
            }
        }
    }
}


/// Return the user's config as a BTreeMap.
pub fn read_config(config_file_path: String)
        -> Result<BTreeMap<String, ConfigCategory>, Box<error::Error>> {
    let f = File::open(&config_file_path)?;
    let reader = BufReader::new(f);

    let config: BTreeMap<String, ConfigCategory> = serde_yaml::from_reader(reader)?;
    return Ok(config);
}


/// Save the data from the given BTreeMap to the user's config file.
pub fn write_config(config_file_path: String, config: BTreeMap<String, ConfigCategory>) {
    let f = OpenOptions::new().write(true).create(true).truncate(true).open(
        &config_file_path);
    let error_msg = format!("Could not write {}", &config_file_path);
    let mut f = f.expect(&error_msg);
    let yaml = serde_yaml::to_string(&config).unwrap();

    f.write_all(&yaml.into_bytes()).expect("Could not write {}");
}


#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "model")]
pub enum ConfigCategory {
    Even {
        choices: Vec<String>
    },
    Gaussian {
        #[serde(default = "default_stddev_scaling_factor")]
        stddev_scaling_factor: f64,
        choices: Vec<String>
    },
    Lottery {
        choices: Vec<LotteryChoice>
    },
    Weighted {
        choices: Vec<WeightedChoice>
    }
}


#[derive(PartialEq, Serialize, Deserialize)]
pub struct LotteryChoice {
    name: String,
    #[serde(default = "default_weight")]
    tickets: u64,
    #[serde(default = "default_weight")]
    weight: u64,
}


#[derive(PartialEq, Serialize, Deserialize)]
pub struct WeightedChoice {
    name: String,
    #[serde(default = "default_weight")]
    weight: u64,
}


/// Define the default for the stddev_scaling_factor setting as 3.0.
fn default_stddev_scaling_factor() -> f64 {
    return 3.0;
}


/// Define the default for the weight setting as 1.
fn default_weight() -> u64 {
    return 1;
}


#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(default_stddev_scaling_factor(), 3.0);
        assert_eq!(default_weight(), 1);
    }

    #[test]
    fn test_get_consent() {
        let tests = [
            (String::from("y"), true), (String::from("Y"), true), (String::from("\n"), true),
            (String::from("f"), false), (String::from("F"), false),
            (String::from("anything else"), false)];

        for (input, expected_output) in tests.iter() {
            let output = Vec::new();
            let mut engine = Engine{input: input.as_bytes(), output: output,
                                    rng: rand::rngs::SmallRng::seed_from_u64(42)};

            assert_eq!(engine.get_consent("do you want this"), *expected_output);

            let output = String::from_utf8(engine.output).expect("Not UTF-8");
            assert_eq!(output, "Choice is do you want this. Accept? (Y/n) ");
        }
    }

    #[test]
    fn test_pick_even() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine{input: input.as_bytes(), output: output,
                                rng: rand::rngs::SmallRng::seed_from_u64(42)};
        let choices = vec![String::from("this"), String::from("that"), String::from("the other")];

        engine.pick_even(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) ");
    }
}
