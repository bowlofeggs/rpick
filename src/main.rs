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

use std::collections::BTreeMap;
use std::io;

use clap::Parser;

use rpick::config;
use rpick::engine::PickError;

mod cli;

const CONFIG_FILE: &str = "rpick.yml";

#[derive(Parser)]
struct CliArgs {
    /// The category you wish to pick from.
    category: String,
    #[clap(short, long, env = "RPICK_CONFIG")]
    /// A path to the config file you wish to use.
    config: Option<String>,
    #[clap(short, long)]
    /// Print more information about the pick.
    verbose: bool,
}

/// Get the `ConfigCategory` to feed to the engine.
fn get_config_category<'a>(
    config: &'a mut BTreeMap<String, config::ConfigCategory>,
    category: &str,
    ad_hoc_category: &'a mut Option<config::ConfigCategory>,
) -> Result<&'a mut config::ConfigCategory, PickError> {
    if category == "-" {
        let choices = io::stdin()
            .lines()
            .collect::<Result<Vec<String>, io::Error>>()
            .unwrap();
        let config_category = config::ConfigCategory::Even { choices };
        *ad_hoc_category = Some(config_category);
        Ok(ad_hoc_category.as_mut().unwrap())
    } else {
        config
            .get_mut(category)
            .ok_or_else(|| PickError::CategoryNotFound(category.to_owned()))
    }
}

fn main() {
    let args = CliArgs::parse();
    let config_path = get_config_file_path(&args);
    let config = match rpick::config::read_config(&config_path) {
        Ok(config) => config,
        Err(error) => {
            println!("Error reading config file at {}: {}", config_path, error);
            std::process::exit(1);
        }
    };
    let mut config = config;
    let ui = cli::Cli::new(args.verbose);

    let mut engine = rpick::engine::Engine::new(&ui);
    let mut possible_dynamic_category = None;
    let config_category = match get_config_category(
        &mut config,
        &args.category[..],
        &mut possible_dynamic_category,
    ) {
        Ok(config_category) => config_category,
        Err(error) => {
            println!("{}", error);
            std::process::exit(1);
        }
    };
    engine.pick(config_category);
    if let Err(e) = rpick::config::write_config(&config_path, config) {
        println!("{}", e);
        std::process::exit(1);
    }
}

/// Return the path to the user's config file.
///
/// If the config flag is set in the given CLI args, that path is used. Otherwise, the default
/// config name (CONFIG_FILE) is appended to the user's home config directory to form the path.
fn get_config_file_path(args: &CliArgs) -> String {
    match &args.config {
        Some(config) => config.clone(),
        None => {
            let config_dir = dirs_next::config_dir().expect("Unable to find config dir.");
            let config_file = config_dir.join(CONFIG_FILE);
            String::from(config_file.to_str().expect("Unable to determine config."))
        }
    }
}
