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
/// Assert correct operation of the lottery model.
use std::collections::{BTreeMap, HashSet};

use rpick::config::ConfigCategory;

const CONFIG: &str = "
---
lottery:
  model: lottery
  choices:
    - name: option 1
      reset: 0
      weight: 1
      tickets: 1
    - name: option 2
      reset: 1
      weight: 2
      tickets: 1
    - name: option 3
      reset: 2
      weight: 3
      tickets: 1
    - name: option 4
      weight: 4
      # This one should never get picked
      tickets: 0
";

#[test]
// Assert correct behavior with an lottery model config
fn pick() {
    let (stdout, config_contents) =
        super::test_rpick_with_config(CONFIG, &mut ["lottery"], "y\n", true);

    // Assert that the chosen item was a member of the config. Note that "option 4" is not listed
    // here, though it is in the config, since it has 0 tickets and should never be chosen.
    let expected_values: HashSet<&'static str> = ["option 1", "option 2", "option 3"]
        .iter()
        .cloned()
        .collect();
    let pick = super::get_pick(&stdout);
    assert!(expected_values.contains(pick.as_str()));
    // Assert that the lottery model removes the tickets on the picked item, and gives more tickets
    // to the ones that weren't picked.
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Lottery { choices } = &mut expected_config.get_mut("lottery").unwrap() {
        for choice in choices.iter_mut() {
            if choice.name == pick {
                choice.tickets = choice.reset;
            } else {
                choice.tickets += choice.weight;
            }
        }
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}
