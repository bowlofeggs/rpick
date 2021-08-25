/*
 * Copyright © 2020 Randy Barlow
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
/// Test the even model.
use std::collections::{BTreeMap, HashSet};

use rpick::config::ConfigCategory;

const CONFIG: &str = "
---
even:
  model: even
  choices:
    - option 1
    - option 2
    - option 3
";

// Assert correct behavior with an even distribution model config
#[test]
fn pick() {
    let (stdout, config_contents) =
        super::test_rpick_with_config(CONFIG, &mut vec!["even"], "y\n", true);

    let expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(CONFIG).expect("Could not parse yaml");
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    // The even config does not modify the config file
    assert_eq!(parsed_config, expected_config);
    // Assert that the chosen item was a member of the config
    let expected_values: HashSet<&'static str> = ["option 1", "option 2", "option 3"]
        .iter()
        .cloned()
        .collect();
    let pick = super::get_pick(&stdout);
    assert!(expected_values.contains(pick.as_str()));
}
