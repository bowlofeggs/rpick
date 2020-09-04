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
/// The tests in this module assert correct error handling.


const CATEGORY_NOT_FOUND_CONFIG: &str = "
---
test:
  model: gaussian
  choices:
    - option 1
    - option 2
    - option 3
";

#[test]
// The user should get a useful error message if the requested category does not exist.
fn category_not_found() {
    let expected_output =
        "Category does_not_exist not found in config.\n";

    let (stdout, config_contents) = super::test_rpick_with_config(
        CATEGORY_NOT_FOUND_CONFIG, &mut vec!["does_not_exist"], "", false);

    assert_eq!(stdout, expected_output);
    // Since the category didn't exist, rpick should not have changed the file.
    assert_eq!(config_contents, CATEGORY_NOT_FOUND_CONFIG);
}

#[test]
// Assert correct behavior when the config file is not found.
fn config_not_found() {
    let expected_output = "Error reading config file at /does/not/exist: No such file or \
                          directory (os error 2)\n";

    let stdout = super::test_rpick(&["-c", "/does/not/exist", "test"], "", false);

    assert_eq!(stdout, expected_output);
}
