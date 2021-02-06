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
/// Test the cli with the Gaussian model.
use std::collections::{BTreeMap, HashSet};

use rpick::config::ConfigCategory;

const CONFIG: &str = "
---
gaussian:
  model: gaussian
  choices:
    - option 1
    - option 2
    - option 3
";

#[test]
// Assert correct behavior with an gaussian model config
fn pick() {
    let (stdout, config_contents) =
        super::test_rpick_with_config(CONFIG, &mut vec!["gaussian"], "y\n", true);

    // Assert that the chosen item was a member of the config
    let expected_values: HashSet<&'static str> = ["option 1", "option 2", "option 3"]
        .iter()
        .cloned()
        .collect();
    let pick = super::get_pick(&stdout);
    assert_eq!(expected_values.contains(pick.as_str()), true);
    // Assert that the gaussian model moves the picked item into last place
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Gaussian {
        choices,
        stddev_scaling_factor: _,
    } = &mut expected_config.get_mut("gaussian").unwrap()
    {
        let index = choices.iter().position(|x| x == pick.as_str()).unwrap();
        choices.remove(index);
        choices.push(pick);
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}
