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

use structopt::StructOpt;

mod cli;

const CONFIG_FILE: &str = "rpick.yml";

#[derive(StructOpt)]
struct CliArgs {
    /// The category you wish to pick from.
    category: String,
    #[structopt(short, long, env = "RPICK_CONFIG")]
    /// A path to the config file you wish to use.
    config: Option<String>,
    #[structopt(short, long)]
    /// Print more information about the pick.
    verbose: bool,
}

fn main() {
    let args = CliArgs::from_args();
    let config_path = get_config_file_path(&args);
    let config = rpick::config::read_config(&config_path);
    match config {
        Ok(config) => {
            let mut config = config;
            let ui = cli::Cli::new(args.verbose);

            let mut engine = rpick::engine::Engine::new(&ui);
            match engine.pick(&mut config, args.category) {
                Ok(_) => match rpick::config::write_config(&config_path, config) {
                    Ok(_) => {}
                    Err(error) => {
                        println!("{}", error);
                        std::process::exit(1);
                    }
                },
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            println!("Error reading config file at {}: {}", config_path, error);
            std::process::exit(1);
        }
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
