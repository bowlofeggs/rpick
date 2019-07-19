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
use std::{error, fmt};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use rand::distributions::{Distribution,Normal};
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};


/// The rpick Engine object allows you to write your own rpick interface.
///
/// # Attributes
///
/// * `input` - This must be an object that implements the [`BufRead`] trait, and is used to
///             receive a y or n answer from a user, when prompted for whether they accept some
///             input. The rpick CLI sets this to stdin, for example.
/// * `output` - This must be an object that implements the [`Write`] trait. It is used to
///              prompt the user to accept a choice.
/// * `rng` - This must be a random number generator that implements the [`rand::RngCore`]
///           trait.
/// * `rejected_choices` - A list of choices the user has rejected. We maintain this so we don't
///                        ask again about a choice they've already declined.
pub struct Engine<I, O, R> {
    input: I,
    output: O,
    rng: R,
    rejected_choices: Vec<String>
}


impl<I, O, R> Engine<I, O, R>
where
    I: BufRead,
    O: Write,
    R: rand::RngCore,
{
    /// Instantiate an Engine.
    ///
    /// # Arguments
    ///
    /// * `input` - This must be an object that implements the [`BufRead`] trait, and is used to
    ///             receive a y or n answer from a user, when prompted for whether they accept some
    ///             input. The rpick CLI sets this to stdin, for example.
    /// * `output` - This must be an object that implements the [`Write`] trait. It is used to
    ///              prompt the user to accept a choice.
    /// * `rng` - This must be a random number generator that implements the [`rand::RngCore`]
    ///           trait.
    ///
    /// # Example
    ///
    /// ```
    /// let stdio = std::io::stdin();
    /// let input = stdio.lock();
    /// let output = std::io::stdout();
    ///
    /// let mut engine = rpick::Engine::new(input, output, rand::thread_rng());
    /// ```
    pub fn new(input: I, output: O, rng:R) -> Engine<I, O, R> {
        Engine{input, output, rng, rejected_choices: Vec::new()}
    }

    /// Pick an item from the [`ConfigCategory`] referenced by the given `category`.
    ///
    /// # Arguments
    ///
    /// * `config` - A mapping of category names to [`ConfigCategory`] objects, which contain the
    ///              parameters which should be used for the pick.
    /// * `category` - The category you wish to choose from.
    ///
    /// # Returns
    ///
    /// This will return the chosen item.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use rand::SeedableRng;
    ///
    /// let input = String::from("y");
    /// let output = Vec::new();
    /// // We need to seed the engine differently for 32-bit architectures than for 64-bit so that
    /// // they each pick the same result for this example.
    /// #[cfg(target_pointer_width = "64")]
    /// let mut engine = rpick::Engine::new(input.as_bytes(), output,
    ///                                     rand::rngs::SmallRng::seed_from_u64(42));
    /// #[cfg(target_pointer_width = "32")]
    /// let mut engine = rpick::Engine::new(input.as_bytes(), output,
    ///                                     rand::rngs::SmallRng::seed_from_u64(32));
    /// let choices = vec![String::from("this"), String::from("that"), String::from("the other")];
    /// let category = rpick::ConfigCategory::Even{choices: choices};
    /// let mut config = BTreeMap::new();
    /// config.insert("things".to_string(), category);
    ///
    /// let choice = engine.pick(&mut config, "things".to_string()).expect("unexpected");
    ///
    /// assert_eq!(choice, "this");
    /// ```
    pub fn pick(&mut self, config: &mut BTreeMap<String, ConfigCategory>, category: String)
            -> Result<String, Box<dyn error::Error>> {
        let config_category = config.get_mut(&category[..]);
        match config_category {
            Some(category) => {
                match category {
                    ConfigCategory::Even { choices } => {
                        Ok(self.pick_even(choices))
                    }
                    ConfigCategory::Gaussian { choices, stddev_scaling_factor } => {
                        Ok(self.pick_gaussian(choices, *stddev_scaling_factor))
                    }
                    ConfigCategory::Lottery { choices } => {
                        Ok(self.pick_lottery(choices))
                    }
                    ConfigCategory::LRU { choices } => {
                        Ok(self.pick_lru(choices))
                    }
                    ConfigCategory::Weighted { choices } => {
                        Ok(self.pick_weighted(choices))
                    }
                }
            }
            None => {
                Err(Box::new(ValueError::new(
                    format!("Category {} not found in config.", category))))
            }
        }
    }

    /// Prompt the user for consent for the given choice, returning a bool true if they accept the
    /// choice, or false if they do not.
    fn get_consent(&mut self, choice: &str, num_choices: usize) -> bool {
        if self.rejected_choices.contains(&choice.to_string()) {
            return false;
        }

        write!(self.output, "Choice is {}. Accept? (Y/n) ", choice).expect(
            "Could not write to output");
        self.output.flush().unwrap();
        let line1 = self.input.by_ref().lines().next().unwrap().unwrap();
        if ["", "y", "Y"].contains(&line1.as_str()) {
            return true;
        }
        if self.rejected_choices.len() + 1 >= num_choices {
            // The user has now rejected all choices. Rather than looping forever, we can just clear
            // our rejected choices list and let them go through them all again.
            self.rejected_choices = Vec::new();
            writeln!(self.output, "🤨").expect("Could not write to output");
        }
        self.rejected_choices.push(choice.to_string());
        false
    }

    /// Use an even distribution random model to pick from the given choices.
    fn pick_even(&mut self, choices: &[String]) -> String {
        let choices = choices.iter().map(|x| (x, 1)).collect::<Vec<_>>();

        loop {
            let choice = choices.choose_weighted(&mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(choice, choices.len()) {
                return choice.clone();
            }
        }
    }


    /// Run the gaussian model for the given choices and standard deviation scaling factor. When the
    /// user accepts a choice, move that choice to end of the choices Vector and return.
    fn pick_gaussian(&mut self, choices: &mut Vec<String>, stddev_scaling_factor: f64) -> String {
        let stddev = (choices.len() as f64) / stddev_scaling_factor;
        let normal = Normal::new(0.0, stddev);
        let mut index;

        loop {
            index = normal.sample(&mut self.rng).abs() as usize;
            if let Some(value) = choices.get(index) {
                if self.get_consent(&value[..], choices.len()) {
                    break;
                }
            }
        }

        let value = choices.remove(index);
        choices.push(value.clone());
        value
    }

    /// Run the LRU model for the given choices. When the user accepts a choice, move that choice to
    /// the end of the choices Vector and return.
    fn pick_lru(&mut self, choices: &mut Vec<String>) -> String {
        for (index, choice) in choices.iter().enumerate() {
            if self.get_consent(&choice[..], choices.len()) {
                let chosen = choices.remove(index);
                choices.push(chosen.clone());
                return chosen;
            }
        }
        // If we've gotten here, the user hasn't made a choice. So… let's do it again!
        self.pick_lru(choices)
    }

    /// Run the lottery model for the given choices.
    fn pick_lottery(&mut self, choices: &mut Vec<LotteryChoice>) -> String {
        let weighted_choices = choices.iter().enumerate().map(
            |x| ((x.0, &x.1.name), x.1.tickets)).collect::<Vec<_>>();

        let index = loop {
            let (index, choice) = weighted_choices.choose_weighted(
                &mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(&choice[..], choices.len()) {
                break index;
            }
        };

        for choice in choices.iter_mut() {
            choice.tickets += choice.weight;
        }
        choices[index].tickets = 0;
        choices[index].name.clone()
    }


    /// Run the weighted model for the given choices.
    fn pick_weighted(&mut self, choices: &[WeightedChoice]) -> String {
        let choices = choices.iter().map(|x| (&x.name, x.weight)).collect::<Vec<_>>();

        loop {
            let choice = choices.choose_weighted(&mut self.rng, |item| item.1).unwrap().0;

            if self.get_consent(&choice[..], choices.len()) {
                return choice.clone();
            }
        }
    }
}


/// Returned in the event that an invalid parameter was used in the API.
#[derive(Debug)]
struct ValueError {
    message: String
}


impl ValueError {
    /// Construct a new ValueError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to accompany the ValueError.
    fn new(message: String) -> ValueError {
        ValueError{message}
    }
}


impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


impl error::Error for ValueError {}


/// Return the user's config as a BTreeMap.
///
/// # Arguments
///
/// * `config_file_path` - A filesystem path to a YAML file that should be read.
///
/// # Returns
///
/// Returns a mapping of YAML to [`ConfigCategory`]'s, or an Error.
pub fn read_config(config_file_path: &str)
        -> Result<BTreeMap<String, ConfigCategory>, Box<error::Error>> {
    let f = File::open(&config_file_path)?;
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
pub fn write_config(config_file_path: &str, config: BTreeMap<String, ConfigCategory>) 
        -> Result<(), Box<error::Error>> {
    let mut f = OpenOptions::new().write(true).create(true).truncate(true).open(
        &config_file_path)?;
    let yaml = serde_yaml::to_string(&config).unwrap();

    f.write_all(&yaml.into_bytes())?;
    Ok(())
}


/// A category of items that can be chosen from.
///
/// Each variant of this Enum maps to one of the supported algorithms.
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "model")]
pub enum ConfigCategory {
    /// The Even variant picks from its choices with even distribution.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Even {
        choices: Vec<String>
    },
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
        choices: Vec<String>
    },
    /// The LRU variant picks the Least Recently Used item from the list of choices. The least
    /// recently used choice is found at the beginning of the list. Once a choice has been
    /// accepted, it is moved to the end of the list.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    #[serde(rename = "lru")]
    LRU {
        choices: Vec<String>
    },
    /// The Lottery variant uses a weighted distribution to pick items, with each items chances
    /// being tied to how many tickets it has. When a choice is accepted, that choice's ticket
    /// count is set to 0, and every choice not chosen receives its weight in additional tickets.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Lottery {
        choices: Vec<LotteryChoice>
    },
    /// The Weighted variant is a simple weighted distribution.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    Weighted {
        choices: Vec<WeightedChoice>
    }
}


/// Represents an individual choice for the lottery model.
///
/// # Attributes
///
/// * `name` - The name of the choice.
/// * `tickets` - The current number of tickets the choice has.
/// * `weight` - The number of tickets that will be added to `tickets` each time this choice is not
///   picked.
#[derive(Debug)]
#[derive(PartialEq, Serialize, Deserialize)]
pub struct LotteryChoice {
    name: String,
    #[serde(default = "default_weight")]
    tickets: u64,
    #[serde(default = "default_weight")]
    weight: u64,
}


/// Represents an individual choice for the weighted model.
///
/// # Attributes
///
/// * `name` - The name of the choice
/// * `weight` - How much chance this choice has of being chosen, relative to the other choices.
#[derive(PartialEq, Serialize, Deserialize)]
pub struct WeightedChoice {
    name: String,
    #[serde(default = "default_weight")]
    weight: u64,
}


/// Define the default for the stddev_scaling_factor setting as 3.0.
fn default_stddev_scaling_factor() -> f64 {
    3.0
}


/// Define the default for the weight setting as 1.
fn default_weight() -> u64 {
    1
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
            let mut engine = Engine::new(input.as_bytes(), output,
                                         rand::rngs::SmallRng::seed_from_u64(42));

            assert_eq!(engine.get_consent("do you want this", 2), *expected_output);

            let output = String::from_utf8(engine.output).expect("Not UTF-8");
            assert_eq!(output, "Choice is do you want this. Accept? (Y/n) ");
            let mut expected_rejected_choices: Vec<String> = Vec::new();
            if !expected_output {
                expected_rejected_choices = vec![String::from("do you want this")];
            }
            assert_eq!(engine.rejected_choices, expected_rejected_choices);
        }
    }

    #[test]
    fn test_pick() {
        let input = String::from("N\ny");
        let output = Vec::new();
        // We need to seed the engine differently for 32-bit architectures than for 64-bit so that
        // they each pick the same result for this test.
        #[cfg(target_pointer_width = "64")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(42));
        #[cfg(target_pointer_width = "32")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(32));
        let choices = vec![String::from("this"), String::from("that"), String::from("the other")];
        let category = ConfigCategory::Even{choices};
        let mut config = BTreeMap::new();
        config.insert("things".to_string(), category);

        let choice = engine.pick(&mut config, "things".to_string()).expect("unexpected");

        assert_eq!(choice, "the other");
        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) Choice is the other. Accept? (Y/n) ");
    }

    #[test]
    fn test_pick_nonexistant_category() {
        let input = String::from("N\ny");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(42));
        let choices = vec![String::from("this"), String::from("that"), String::from("the other")];
        let category = ConfigCategory::Even{choices};
        let mut config = BTreeMap::new();
        config.insert("things".to_string(), category);

        match engine.pick(&mut config, "does not exist".to_string()) {
            Ok(_) => {
                panic!("The non-existant category should have returned an error.");
            },
            Err(error) => {
                assert_eq!(format!("{}", error), "Category does not exist not found in config.");
            }
        }
    }

    #[test]
    fn test_pick_even() {
        let input = String::from("y");
        let output = Vec::new();
        // We need to seed the engine differently for 32-bit architectures than for 64-bit so that
        // they each pick the same result for this test.
        #[cfg(target_pointer_width = "64")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(1));
        #[cfg(target_pointer_width = "32")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(5));
        let choices = vec![String::from("this"), String::from("that"), String::from("the other")];

        let result = engine.pick_even(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is the other. Accept? (Y/n) ");
        assert_eq!(result, "the other");
    }

    #[test]
    fn test_pick_gaussian() {
        let input = String::from("y");
        let output = Vec::new();
        // We need to seed the engine differently for 32-bit architectures than for 64-bit so that
        // they each pick the same result for this test.
        #[cfg(target_pointer_width = "64")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(1));
        #[cfg(target_pointer_width = "32")]
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(2));
        let mut choices = vec![
            String::from("this"), String::from("that"), String::from("the other")];

        let result = engine.pick_gaussian(&mut choices, 3.0);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is that. Accept? (Y/n) ");
        assert_eq!(result, "that");
        assert_eq!(choices,
                   vec![String::from("this"), String::from("the other"), String::from("that")]);
    }

    #[test]
    fn test_pick_lru() {
        // The user says no to the first one and yes to the second.
        let input = String::from("n\ny");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(1));
        let mut choices = vec![
            String::from("this"), String::from("that"), String::from("the other")];

        let result = engine.pick_lru(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) Choice is that. Accept? (Y/n) ");
        assert_eq!(result, "that");
        assert_eq!(choices,
                   vec![String::from("this"), String::from("the other"), String::from("that")]);
    }

    #[test]
    fn test_pick_lottery() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(2));
        let mut choices = vec![
            LotteryChoice{name: "this".to_string(), tickets: 1, weight: 1},
            LotteryChoice{name: "that".to_string(), tickets: 2, weight: 4},
            LotteryChoice{name: "the other".to_string(), tickets:3, weight: 9}];

        let result = engine.pick_lottery(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is the other. Accept? (Y/n) ");
        assert_eq!(result, "the other");
        assert_eq!(
            choices,
            vec![
                LotteryChoice{name: "this".to_string(), tickets: 2, weight: 1},
                LotteryChoice{name: "that".to_string(), tickets: 6, weight: 4},
                LotteryChoice{name: "the other".to_string(), tickets: 0, weight: 9}]);
    }

    #[test]
    fn test_pick_weighted() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output,
                                     rand::rngs::SmallRng::seed_from_u64(3));
        let choices = vec![
            WeightedChoice{name: "this".to_string(), weight: 1},
            WeightedChoice{name: "that".to_string(), weight: 4},
            WeightedChoice{name: "the other".to_string(), weight: 9}];

        let result = engine.pick_weighted(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is that. Accept? (Y/n) ");
        assert_eq!(result, "that");
    }
}
