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
/// Assert correct operation of the lru model.
use std::collections::BTreeMap;

use rpick::config::ConfigCategory;

const CONFIG: &str = "
---
lru:
  model: lru
  choices:
    - option 1
    - option 2
    - option 3
";

#[test]
// Assert correct behavior with an lru model config
fn pick() {
    let (stdout, config_contents) =
        super::test_rpick_with_config(CONFIG, &mut vec!["lru"], "y\n", true);

    // Assert that the chosen item was a member of the config
    assert_eq!(super::get_pick(&stdout), "option 1");
    // Assert that the lru model moves the picked item into last place
    let mut expected_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(CONFIG).expect("Could not parse yaml");
    if let ConfigCategory::Lru { choices } = &mut expected_config.get_mut("lru").unwrap() {
        let pick = choices.remove(0);
        choices.push(pick);
    }
    let parsed_config: BTreeMap<String, ConfigCategory> =
        serde_yaml::from_str(&config_contents).expect("Could not parse yaml");
    assert_eq!(parsed_config, expected_config);
}
