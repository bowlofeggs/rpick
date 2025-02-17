/* Copyright © 2019-2021, 2025 Randy Barlow
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
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use rand::SeedableRng;
//!
//! /// You need to define an interface. rpick will use this interface to interact with you during
//! /// picks.
//! struct Interface {};
//!
//! impl rpick::ui::Ui for Interface {
//!     fn call_display_table(&self) -> bool { false }
//!
//!     fn display_table(&self, table: &rpick::ui::Table) {}
//!
//!     fn info(&self, message: &str) { println!("{}", message); }
//!
//!     fn prompt_choice(&self, choice: &str) -> bool {
//!         println!("{}", choice);
//!         true
//!     }
//! }
//!
//! let ui = Interface{};
//! let mut engine = rpick::engine::Engine::new(&ui);
//! // For the sake of this example, let's override the PRNG with a seeded PRNG so the assertion
//! // works as expected at the end. You most likely do not want to do this in practice as it takes
//! // the randomness out of the system.
//! engine.set_rng(rand::rngs::SmallRng::seed_from_u64(37));
//! let choices = vec![String::from("this"), String::from("that"),
//!                    String::from("the other")];
//! let category = rpick::config::ConfigCategory::Even{choices: choices};
//! let mut config = BTreeMap::new();
//! config.insert("things".to_string(), category);
//!
//! let choice = engine.pick(&mut config, "things").unwrap();
//!
//! // 32-bit architectures have different PRNG results than 64-bit architectures, so we will
//! // only run this assertion on 64-bit systems.
//! #[cfg(target_pointer_width = "64")]
//! assert_eq!(choice, "that");
//! ```
pub mod config;
pub mod engine;
pub mod ui;
