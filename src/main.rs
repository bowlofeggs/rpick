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

use std::io;

use structopt::StructOpt;


const CONFIG_FILE: &str = "rpick.yml";


#[derive(StructOpt)]
struct Cli {
    /// The category you wish to pick from.
    category: String,
}


fn main() {
    let args = Cli::from_args();
    let config_path = get_config_file_path();
    let config = rpick::read_config(&config_path);
    match config {
        Ok(config) => {
            let mut config = config;
            let stdio = io::stdin();
            let input = stdio.lock();
            let output = io::stdout();

            let mut engine = rpick::Engine::new(input, output, rand::thread_rng());
            match engine.pick(&mut config, args.category) {
                Ok(_) => {
                    rpick::write_config(&config_path, config);
                },
                Err(error) => {
                    panic!(format!("{}", error));
                }
            }
        },
        Err(error) => {
            panic!(format!("{}", error));
        }
    }
}


/// Return the path to the user's config file.
fn get_config_file_path() -> String {
    let config_dir = dirs::config_dir().expect("Unable to find config dir.");
    let config_file = config_dir.join(CONFIG_FILE);
    return String::from(config_file.to_str().expect("Unable to determine config."));
}
