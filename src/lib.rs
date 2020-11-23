/* Copyright © 2019-2020 Randy Barlow
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
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::{error, fmt};

#[macro_use]
extern crate prettytable;
use prettytable::{format, Cell, Table};
use rand::seq::SliceRandom;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use statrs::distribution::Univariate;

/// The rpick Engine object allows you to write your own rpick interface.
///
/// # Attributes
///
/// * `color` - Whether to use color when printing to output. Defaults to false.
/// * `verbose` - Whether to print more info when picking choices. Defaults to false.
/// * `input` - This must be an object that implements the [`BufRead`] trait, and is used to
///             receive a y or n answer from a user, when prompted for whether they accept some
///             input. The rpick CLI sets this to stdin, for example.
/// * `output` - This must be an object that implements the [`Write`] trait. It is used to
///              prompt the user to accept a choice.
/// * `rng` - This must be a random number generator that implements the [`rand::RngCore`]
///           trait.
pub struct Engine<I, O, R> {
    pub color: bool,
    pub verbose: bool,
    input: I,
    output: O,
    rng: R,
}

impl<'a, I, O, R> Engine<I, O, R>
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
    pub fn new(input: I, output: O, rng: R) -> Engine<I, O, R> {
        Engine {
            input,
            output,
            rng,
            color: false,
            verbose: false,
        }
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
    /// // 32-bit architectures have different PRNG results than 64-bit architectures, so we will
    /// // only run this example on 64-bit systems.
    /// #[cfg(target_pointer_width = "64")]
    /// {
    ///     use std::collections::BTreeMap;
    ///
    ///     use rand::SeedableRng;
    ///
    ///     let input = String::from("y");
    ///     let output = Vec::new();
    ///     let mut engine = rpick::Engine::new(input.as_bytes(), output,
    ///                                         rand::rngs::SmallRng::seed_from_u64(42));
    ///     let choices = vec![String::from("this"), String::from("that"),
    ///                        String::from("the other")];
    ///     let category = rpick::ConfigCategory::Even{choices: choices};
    ///     let mut config = BTreeMap::new();
    ///     config.insert("things".to_string(), category);
    ///
    ///     let choice = engine.pick(&mut config, "things".to_string()).expect("unexpected");
    ///
    ///     assert_eq!(choice, "that");
    /// }
    /// ```
    pub fn pick(
        &mut self,
        config: &mut BTreeMap<String, ConfigCategory>,
        category: String,
    ) -> Result<String, Box<dyn error::Error>> {
        let config_category = config.get_mut(&category[..]);
        match config_category {
            Some(category) => match category {
                ConfigCategory::Even { choices } => Ok(self.pick_even(choices)),
                ConfigCategory::Gaussian {
                    choices,
                    stddev_scaling_factor,
                } => Ok(self.pick_gaussian(choices, *stddev_scaling_factor)),
                ConfigCategory::Inventory { choices } => Ok(self.pick_inventory(choices)),
                ConfigCategory::Lottery { choices } => Ok(self.pick_lottery(choices)),
                ConfigCategory::LRU { choices } => Ok(self.pick_lru(choices)),
                ConfigCategory::Weighted { choices } => Ok(self.pick_weighted(choices)),
            },
            None => Err(Box::new(ValueError::new(format!(
                "Category {} not found in config.",
                category
            )))),
        }
    }

    /// Express disapproval to the user.
    fn express_disapproval(&mut self) {
        writeln!(self.output, "🤨").expect("Could not write to output");
    }

    /// Prompt the user for consent for the given choice, returning a bool true if they accept the
    /// choice, or false if they do not.
    fn get_consent(&mut self, choice: &str) -> bool {
        write!(self.output, "Choice is {}. Accept? (Y/n) ", choice)
            .expect("Could not write to output");
        self.output.flush().unwrap();
        let line1 = self.input.by_ref().lines().next().unwrap().unwrap();
        if ["", "y", "Y"].contains(&line1.as_str()) {
            return true;
        }
        false
    }

    /// Use an even distribution random model to pick from the given choices.
    fn pick_even(&mut self, choices: &[String]) -> String {
        let initialize_candidates = || {
            choices
                .iter()
                .enumerate()
                .map(|x| ((x.0, x.1), 1))
                .collect::<Vec<_>>()
        };

        let index = self.pick_weighted_common(&initialize_candidates);

        choices[index].clone()
    }

    /// Run the gaussian model for the given choices and standard deviation scaling factor. When the
    /// user accepts a choice, move that choice to end of the choices Vector and return.
    fn pick_gaussian(&mut self, choices: &mut Vec<String>, stddev_scaling_factor: f64) -> String {
        let mut candidates = choices.clone();
        let mut index;

        loop {
            let stddev = (candidates.len() as f64) / stddev_scaling_factor;
            let normal = Normal::new(0.0, stddev).unwrap();
            index = normal.sample(&mut self.rng).abs() as usize;

            if let Some(value) = candidates.get(index) {
                if self.verbose {
                    self.print_gaussian_chance_table(index, &candidates, stddev);
                }

                if self.get_consent(&value[..]) {
                    index = choices.iter().position(|x| x == value).unwrap();
                    break;
                } else if candidates.len() > 1 {
                    index = candidates.iter().position(|x| x == value).unwrap();
                    candidates.remove(index);
                } else {
                    self.express_disapproval();
                    candidates = choices.clone();
                }
            }
        }

        let value = choices.remove(index);
        choices.push(value.clone());
        value
    }

    /// Run the inventory model for the given choices.
    fn pick_inventory(&mut self, choices: &mut Vec<InventoryChoice>) -> String {
        let initialize_candidates = || {
            choices
                .iter()
                .enumerate()
                .filter(|x| x.1.tickets > 0)
                .map(|x| ((x.0, &x.1.name), x.1.tickets))
                .collect::<Vec<_>>()
        };

        let index = self.pick_weighted_common(&initialize_candidates);

        choices[index].tickets -= 1;
        choices[index].name.clone()
    }

    /// Run the LRU model for the given choices. When the user accepts a choice, move that choice to
    /// the end of the choices Vector and return.
    fn pick_lru(&mut self, choices: &mut Vec<String>) -> String {
        for (index, choice) in choices.iter().enumerate() {
            if self.verbose {
                self.print_lru_table(index, &choices);
            }

            if self.get_consent(&choice[..]) {
                let chosen = choices.remove(index);
                choices.push(chosen.clone());
                return chosen;
            }
        }
        self.express_disapproval();
        // If we've gotten here, the user hasn't made a choice. So… let's do it again!
        self.pick_lru(choices)
    }

    /// Run the lottery model for the given choices.
    fn pick_lottery(&mut self, choices: &mut Vec<LotteryChoice>) -> String {
        let initialize_candidates = || {
            choices
                .iter()
                .enumerate()
                .filter(|x| x.1.tickets > 0)
                .map(|x| ((x.0, &x.1.name), x.1.tickets))
                .collect::<Vec<_>>()
        };

        let index = self.pick_weighted_common(&initialize_candidates);

        for choice in choices.iter_mut() {
            choice.tickets += choice.weight;
        }
        choices[index].tickets = 0;
        choices[index].name.clone()
    }

    /// Run the weighted model for the given choices.
    fn pick_weighted(&mut self, choices: &[WeightedChoice]) -> String {
        let initialize_candidates = || {
            choices
                .iter()
                .enumerate()
                .map(|x| ((x.0, &x.1.name), x.1.weight))
                .collect::<Vec<_>>()
        };

        let index = self.pick_weighted_common(&initialize_candidates);

        choices[index].name.clone()
    }

    /// A common weighted choice algorithm used as the core of many models.
    ///
    /// The initialize_candidates() function should return a Vector of 2-tuples. The first element
    /// of the 2-tuple is also a 2-tuple, specifying the original index of the choice and the human
    /// readable name of the choice. The second element of the outer 2-tuple should express the
    /// weight of that choice. For example, if the first choice is "ice cream" and has a weight of
    /// 5, the data structure would look like this: ((0, "ice cream"), 5)
    fn pick_weighted_common(
        &mut self,
        initialize_candidates: &dyn Fn() -> Vec<((usize, &'a String), u64)>,
    ) -> usize {
        let mut candidates = initialize_candidates();

        loop {
            let (index, choice) = candidates
                .choose_weighted(&mut self.rng, |item| item.1)
                .unwrap()
                .0;

            if self.verbose {
                self.print_weighted_chance_table(index, &candidates);
            }

            if self.get_consent(&choice[..]) {
                break index;
            } else if candidates.len() > 1 {
                candidates.remove(candidates.iter().position(|x| (x.0).1 == choice).unwrap());
            } else {
                self.express_disapproval();
                candidates = initialize_candidates();
            }
        }
    }

    /// Print a table to self.output showing the candidates, sorted by chance of being chosen.
    ///
    /// # Arguments
    ///
    /// `index` - The index of the candidate that was chosen. This is used to turn the chosen
    ///     candidate yellow in the table.
    /// `candidates` - A list of the candidates.
    fn print_gaussian_chance_table(&mut self, index: usize, candidates: &[String], stddev: f64) {
        // Let's make a copy of the candidate list so that we can sort it for the table
        // without sorting the real candidate list.
        let candidates = candidates.to_owned();

        let mut table = Table::new();
        table.set_titles(row![c->"Name", r->"Chance"]);
        let distribution = statrs::distribution::Normal::new(0.0, stddev).unwrap();
        let mut total_chance = 0.0;
        for (i, candidate) in candidates.iter().enumerate() {
            // We multiply by 200 here: 100 is for expressing percents to humans, and the factor
            // of 2 is to account for the abs() we use in pick_gaussian(), which causes us to
            // reflect the distribution around the x-axis (thus the chance is this slice of the CDF
            // on both sides of the x-axis, which is the same chance as twice this singular slice).
            let chance: f64 =
                (distribution.cdf((i as f64) + 1.0) - distribution.cdf(i as f64)) * 200.;
            total_chance += chance;
            let mut row = row![];
            let style = if i == index { "bFy" } else { "" };
            row.add_cell(Cell::new(candidate).style_spec(style));
            row.add_cell(Cell::new(&format!("{:>6.2}%", &chance)).style_spec(style));
            table.insert_row(0, row);
        }
        table.add_row(row![b->"Total", b->format!("{:>6.2}%", total_chance)]);

        self.print_table(table);
    }

    /// Print a table to self.output showing the candidates, sorted by chance of being chosen.
    ///
    /// # Arguments
    ///
    /// `index` - The index of the candidate that was chosen. This is used to turn the chosen
    ///     candidate yellow in the table.
    /// `candidates` - A list of the candidates.
    fn print_lru_table(&mut self, index: usize, candidates: &[String]) {
        // Filter out candidates that have already been rejected by the user.
        let candidates = candidates
            .iter()
            .enumerate()
            .filter(|(i, _)| i >= &index)
            .map(|x| x.1)
            .collect::<Vec<_>>();

        let mut table = Table::new();
        table.set_titles(row![c->"Name"]);
        for (i, candidate) in candidates.iter().rev().enumerate() {
            let mut row = row![];
            let style = if i == candidates.len() - 1 { "bFy" } else { "" };
            row.add_cell(Cell::new(candidate).style_spec(style));
            table.add_row(row);
        }

        self.print_table(table);
    }

    /// Print the given table.
    ///
    /// If the user has requested colors and we have a terminal capable of colors, print the table
    /// using the table's print_term method. Otherwise, print the table to self.output without
    /// colors.
    ///
    /// # Arguments
    ///
    /// `table` - The Table we wish to print.
    fn print_table(&mut self, mut table: Table) {
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        writeln!(&mut self.output).expect("Could not write to output");
        match (
            self.color,
            term::terminfo::TerminfoTerminal::new(&mut self.output),
        ) {
            (true, Some(mut term)) => {
                table
                    .print_term(&mut term)
                    .expect("Could not print to terminal");
            }
            _ => {
                table
                    .print(&mut self.output)
                    .expect("Could not print to terminal");
            }
        }
        writeln!(&mut self.output).expect("Could not write to output");
    }

    /// Print a table to self.output showing the candidates, sorted by chance of being chosen.
    ///
    /// # Arguments
    ///
    /// `index` - The index of the candidate that was chosen. This is used to turn the chosen
    ///     candidate yellow in the table.
    /// `candidates` - A list of the candidates.
    fn print_weighted_chance_table(
        &mut self,
        index: usize,
        candidates: &[((usize, &'a String), u64)],
    ) {
        // Let's make a copy of the candidate list so that we can sort it for the table
        // without sorting the real candidate list.
        let mut candidates = candidates.to_owned();
        candidates.sort_by_key(|c| c.1);

        let total: u64 = candidates.iter().map(|x| x.1).sum();

        let mut table = Table::new();
        table.set_titles(row![c->"Name", r->"Weight", r->"Chance"]);
        for candidate in candidates.iter() {
            let chance: f64 = (candidate.1 as f64) / (total as f64) * 100.;
            let mut row = row![];
            let style = if (candidate.0).0 == index { "bFy" } else { "" };
            row.add_cell(Cell::new((candidate.0).1).style_spec(style));
            row.add_cell(Cell::new(&candidate.1.to_string()).style_spec(&format!("r{}", style)));
            row.add_cell(Cell::new(&format!("{:>6.2}%", &chance)).style_spec(style));
            table.add_row(row);
        }
        table.add_row(row![b->"Total", br->total, b->"100.00%"]);

        self.print_table(table);
    }
}

/// Returned in the event that an invalid parameter was used in the API.
#[derive(Debug)]
struct ValueError {
    message: String,
}

impl ValueError {
    /// Construct a new ValueError.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to accompany the ValueError.
    fn new(message: String) -> ValueError {
        ValueError { message }
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
pub fn read_config(
    config_file_path: &str,
) -> Result<BTreeMap<String, ConfigCategory>, Box<dyn error::Error>> {
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
pub fn write_config(
    config_file_path: &str,
    config: BTreeMap<String, ConfigCategory>,
) -> Result<(), Box<dyn error::Error>> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config_file_path)?;
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
    /// The LRU variant picks the Least Recently Used item from the list of choices. The least
    /// recently used choice is found at the beginning of the list. Once a choice has been
    /// accepted, it is moved to the end of the list.
    ///
    /// # Attributes
    ///
    /// * `choices` - The list of choices to pick from.
    #[serde(rename = "lru")]
    LRU { choices: Vec<String> },
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InventoryChoice {
    pub name: String,
    #[serde(default = "default_weight")]
    pub tickets: u64,
}

/// Represents an individual choice for the lottery model.
///
/// # Attributes
///
/// * `name` - The name of the choice.
/// * `tickets` - The current number of tickets the choice has.
/// * `weight` - The number of tickets that will be added to `tickets` each time this choice is not
///   picked.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LotteryChoice {
    pub name: String,
    #[serde(default = "default_weight")]
    pub tickets: u64,
    #[serde(default = "default_weight")]
    pub weight: u64,
}

/// Represents an individual choice for the weighted model.
///
/// # Attributes
///
/// * `name` - The name of the choice
/// * `weight` - How much chance this choice has of being chosen, relative to the other choices.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

    const PICK_GAUSSIAN_VERBOSE_EXPECTED_OUTPUT: &str = r"
   Name    |  Chance 
-----------+---------
 the other |   4.28% 
 that      |  27.18% 
 this      |  68.27% 
 Total     |  99.73% 

Choice is that. Accept? (Y/n) ";

    const PICK_INVENTORY_VERBOSE_EXPECTED_OUTPUT: &str = r"
   Name    | Weight |  Chance 
-----------+--------+---------
 that      |      2 |  40.00% 
 the other |      3 |  60.00% 
 Total     |      5 | 100.00% 

Choice is that. Accept? (Y/n) ";

    const PICK_LRU_VERBOSE_EXPECTED_OUTPUT: &str = r"
   Name 
-----------
 the other 
 that 
 this 

Choice is this. Accept? (Y/n) ";

    struct FakeRng(u32);

    /// This allows our tests to have predictable results, and to have the same predictable results
    /// on both 32-bit and 64-bit architectures. This is used for all tests except for the Gaussian
    /// tests, since those do behave differently between 32-bit and 64-bit systems when using this
    /// rng.
    impl rand::RngCore for FakeRng {
        fn next_u32(&mut self) -> u32 {
            self.0 += 1;
            self.0 - 1
        }

        fn next_u64(&mut self) -> u64 {
            self.next_u32() as u64
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            rand_core::impls::fill_bytes_via_next(self, dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    #[test]
    fn test_defaults() {
        assert!((default_stddev_scaling_factor() - 3.0).abs() < 0.000_001);
        assert_eq!(default_weight(), 1);
    }

    #[test]
    fn test_get_consent() {
        let tests = [
            (String::from("y"), true),
            (String::from("Y"), true),
            (String::from("\n"), true),
            (String::from("f"), false),
            (String::from("F"), false),
            (String::from("anything else"), false),
        ];

        for (input, expected_output) in tests.iter() {
            let output = Vec::new();
            let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));

            assert_eq!(engine.get_consent("do you want this"), *expected_output);

            let output = String::from_utf8(engine.output).expect("Not UTF-8");
            assert_eq!(output, "Choice is do you want this. Accept? (Y/n) ");
        }
    }

    #[test]
    fn test_pick() {
        let input = String::from("N\ny");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];
        let category = ConfigCategory::Even { choices };
        let mut config = BTreeMap::new();
        config.insert("things".to_string(), category);

        let choice = engine
            .pick(&mut config, "things".to_string())
            .expect("unexpected");

        assert_eq!(choice, "that");
        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(
            output,
            "Choice is this. Accept? (Y/n) Choice is that. Accept? (Y/n) "
        );
    }

    #[test]
    fn test_pick_nonexistant_category() {
        let input = String::from("N\ny");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];
        let category = ConfigCategory::Even { choices };
        let mut config = BTreeMap::new();
        config.insert("things".to_string(), category);

        match engine.pick(&mut config, "does not exist".to_string()) {
            Ok(_) => {
                panic!("The non-existant category should have returned an error.");
            }
            Err(error) => {
                assert_eq!(
                    format!("{}", error),
                    "Category does not exist not found in config."
                );
            }
        }
    }

    #[test]
    fn test_pick_even() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];

        let result = engine.pick_even(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) ");
        assert_eq!(result, "this");
    }

    #[test]
    fn test_pick_gaussian() {
        let input = String::from("y");
        let output = Vec::new();
        // Unfortunately, the FakeRng we wrote above causes the Gaussian distribution to often
        // pick outside of the distribution for 32-bit values on 64-bit systems. Since it is a
        // u32, this means that the user saying no here will make the implementation loop forever
        // until it hits MAXINT on 64-bit systems. If we made the FakeRng be a 64 bit value, then
        // the test results on 32-bit systems would overflow. Ideally we'd have a better way than
        // the below to get consistent test results between 32-bit and 64-bit systems, but for now
        // this works OK. We seed the engine differently for 32-bit architectures than for
        // 64-bit so that they each pick the same result for this test.
        #[cfg(target_pointer_width = "64")]
        let mut engine = Engine::new(
            input.as_bytes(),
            output,
            rand::rngs::SmallRng::seed_from_u64(1),
        );
        #[cfg(target_pointer_width = "32")]
        let mut engine = Engine::new(
            input.as_bytes(),
            output,
            rand::rngs::SmallRng::seed_from_u64(2),
        );
        let mut choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];

        let result = engine.pick_gaussian(&mut choices, 3.0);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is that. Accept? (Y/n) ");
        assert_eq!(result, "that");
        assert_eq!(
            choices,
            vec![
                String::from("this"),
                String::from("the other"),
                String::from("that")
            ]
        );
    }

    #[test]
    fn test_pick_gaussian_verbose() {
        let input = String::from("y");
        let output = Vec::new();
        // Unfortunately, the FakeRng we wrote above causes the Gaussian distribution to often
        // pick outside of the distribution for 32-bit values on 64-bit systems. Since it is a
        // u32, this means that the user saying no here will make the implementation loop forever
        // until it hits MAXINT on 64-bit systems. If we made the FakeRng be a 64 bit value, then
        // the test results on 32-bit systems would overflow. Ideally we'd have a better way than
        // the below to get consistent test results between 32-bit and 64-bit systems, but for now
        // this works OK. We seed the engine differently for 32-bit architectures than for
        // 64-bit so that they each pick the same result for this test.
        #[cfg(target_pointer_width = "64")]
        let mut engine = Engine::new(
            input.as_bytes(),
            output,
            rand::rngs::SmallRng::seed_from_u64(1),
        );
        #[cfg(target_pointer_width = "32")]
        let mut engine = Engine::new(
            input.as_bytes(),
            output,
            rand::rngs::SmallRng::seed_from_u64(2),
        );
        engine.verbose = true;
        let mut choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];

        let result = engine.pick_gaussian(&mut choices, 3.0);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, PICK_GAUSSIAN_VERBOSE_EXPECTED_OUTPUT);
        assert_eq!(result, "that");
        assert_eq!(
            choices,
            vec![
                String::from("this"),
                String::from("the other"),
                String::from("that")
            ]
        );
    }

    #[test]
    fn test_pick_inventory() {
        let input = String::from("n\nn\nn\ny\n");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let mut choices = vec![
            InventoryChoice {
                name: "this".to_string(),
                tickets: 0,
            },
            InventoryChoice {
                name: "that".to_string(),
                tickets: 2,
            },
            InventoryChoice {
                name: "the other".to_string(),
                tickets: 3,
            },
        ];

        let result = engine.pick_inventory(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(
            output,
            "Choice is that. Accept? (Y/n) Choice is the other. Accept? (Y/n) 🤨\n\
                            Choice is that. Accept? (Y/n) Choice is the other. Accept? (Y/n) "
        );
        assert_eq!(result, "the other");
        assert_eq!(
            choices,
            vec![
                InventoryChoice {
                    name: "this".to_string(),
                    tickets: 0
                },
                InventoryChoice {
                    name: "that".to_string(),
                    tickets: 2
                },
                InventoryChoice {
                    name: "the other".to_string(),
                    tickets: 2
                }
            ]
        );
    }

    #[test]
    fn test_pick_inventory_verbose() {
        let input = String::from("y\n");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        engine.verbose = true;
        let mut choices = vec![
            InventoryChoice {
                name: "this".to_string(),
                tickets: 0,
            },
            InventoryChoice {
                name: "that".to_string(),
                tickets: 2,
            },
            InventoryChoice {
                name: "the other".to_string(),
                tickets: 3,
            },
        ];

        let result = engine.pick_inventory(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, PICK_INVENTORY_VERBOSE_EXPECTED_OUTPUT);
        assert_eq!(result, "that");
        assert_eq!(
            choices,
            vec![
                InventoryChoice {
                    name: "this".to_string(),
                    tickets: 0
                },
                InventoryChoice {
                    name: "that".to_string(),
                    tickets: 1
                },
                InventoryChoice {
                    name: "the other".to_string(),
                    tickets: 3
                }
            ]
        );
    }

    #[test]
    fn test_pick_lru() {
        // The user says no to the first one and yes to the second.
        let input = String::from("n\ny");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let mut choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];

        let result = engine.pick_lru(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(
            output,
            "Choice is this. Accept? (Y/n) Choice is that. Accept? (Y/n) "
        );
        assert_eq!(result, "that");
        assert_eq!(
            choices,
            vec![
                String::from("this"),
                String::from("the other"),
                String::from("that")
            ]
        );
    }

    #[test]
    /// Test pick_lru() with the verbose flag set
    fn test_pick_lru_verbose() {
        // The user says no to the first one and yes to the second.
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        engine.verbose = true;
        let mut choices = vec![
            String::from("this"),
            String::from("that"),
            String::from("the other"),
        ];

        let result = engine.pick_lru(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, PICK_LRU_VERBOSE_EXPECTED_OUTPUT);
        assert_eq!(result, "this");
        assert_eq!(
            choices,
            vec![
                String::from("that"),
                String::from("the other"),
                String::from("this")
            ]
        );
    }

    #[test]
    fn test_pick_lottery() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let mut choices = vec![
            LotteryChoice {
                name: "this".to_string(),
                tickets: 1,
                weight: 1,
            },
            LotteryChoice {
                name: "that".to_string(),
                tickets: 2,
                weight: 4,
            },
            LotteryChoice {
                name: "the other".to_string(),
                tickets: 3,
                weight: 9,
            },
        ];

        let result = engine.pick_lottery(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) ");
        assert_eq!(result, "this");
        assert_eq!(
            choices,
            vec![
                LotteryChoice {
                    name: "this".to_string(),
                    tickets: 0,
                    weight: 1
                },
                LotteryChoice {
                    name: "that".to_string(),
                    tickets: 6,
                    weight: 4
                },
                LotteryChoice {
                    name: "the other".to_string(),
                    tickets: 12,
                    weight: 9
                }
            ]
        );
    }

    /// If the user says no to all the choices, rpick should print out an emoji and start over.
    /// There was previously a bug where the pick would loop forever if one of the options had 0
    /// chance of being picked.
    #[test]
    fn test_pick_lottery_no_to_all_one_no_chance() {
        let input = String::from("n\nn\nn\ny\n");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let mut choices = vec![
            LotteryChoice {
                name: "this".to_string(),
                tickets: 0,
                weight: 1,
            },
            LotteryChoice {
                name: "that".to_string(),
                tickets: 2,
                weight: 4,
            },
            LotteryChoice {
                name: "the other".to_string(),
                tickets: 3,
                weight: 9,
            },
        ];

        let result = engine.pick_lottery(&mut choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(
            output,
            "Choice is that. Accept? (Y/n) Choice is the other. Accept? (Y/n) 🤨\n\
                            Choice is that. Accept? (Y/n) Choice is the other. Accept? (Y/n) "
        );
        assert_eq!(result, "the other");
        assert_eq!(
            choices,
            vec![
                LotteryChoice {
                    name: "this".to_string(),
                    tickets: 1,
                    weight: 1
                },
                LotteryChoice {
                    name: "that".to_string(),
                    tickets: 6,
                    weight: 4
                },
                LotteryChoice {
                    name: "the other".to_string(),
                    tickets: 0,
                    weight: 9
                }
            ]
        );
    }

    #[test]
    fn test_pick_weighted() {
        let input = String::from("y");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let choices = vec![
            WeightedChoice {
                name: "this".to_string(),
                weight: 1,
            },
            WeightedChoice {
                name: "that".to_string(),
                weight: 4,
            },
            WeightedChoice {
                name: "the other".to_string(),
                weight: 9,
            },
        ];

        let result = engine.pick_weighted(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(output, "Choice is this. Accept? (Y/n) ");
        assert_eq!(result, "this");
    }

    /// There was a bug wherein saying no to all weighted options crashed rpick rather than
    /// expressing disapproval.
    #[test]
    fn test_pick_weighted_no_to_all() {
        let input = String::from("n\nn\nn\ny\n");
        let output = Vec::new();
        let mut engine = Engine::new(input.as_bytes(), output, FakeRng(0));
        let choices = vec![
            WeightedChoice {
                name: "this".to_string(),
                weight: 1,
            },
            WeightedChoice {
                name: "that".to_string(),
                weight: 4,
            },
            WeightedChoice {
                name: "the other".to_string(),
                weight: 9,
            },
        ];

        let result = engine.pick_weighted(&choices);

        let output = String::from_utf8(engine.output).expect("Not UTF-8");
        assert_eq!(
            output,
            "Choice is this. Accept? (Y/n) Choice is that. Accept? (Y/n) \
                            Choice is the other. Accept? (Y/n) 🤨\nChoice is this. Accept? (Y/n) "
        );
        assert_eq!(result, "this");
    }
}
