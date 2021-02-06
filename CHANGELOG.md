# 0.8.0

This is a backwards breaking change in the crate.

* Redesigned the Engine API. It now requires an instance of a struct that implements the
  rpick::ui::UI trait. This trait provides a more natural way to interact with the library than
  streams of bytes.
* The Engine and its Error have been moved into a `rpick::engine` module.
* The configuration structs, enums, and functions have been moved into a `rpick::config` module.
* The Engine `ValueError` struct was replaced by a new `PickError` enum.


# 0.7.2

* [#19](https://github.com/bowlofeggs/rpick/issues/19) Update rand to 0.8.2.
* [#20](https://github.com/bowlofeggs/rpick/issues/20) `cargo test --release` now works.
* [#22](https://github.com/bowlofeggs/rpick/pull/22) Use link time optimization for release builds.


# 0.7.1

* Updated several dependencies.
* Moved the project to [GitHub](https://github.com/bowlofeggs/rpick).


# 0.7.0

* [#43](https://gitlab.com/bowlofeggs/rpick/-/merge_requests/43) There is now a ```--verbose```
  flag.


# 0.6.1

* Update Cargo.lock to get new dependencies.


# 0.6.0

* [#39](https://gitlab.com/bowlofeggs/rpick/-/merge_requests/39): Add a new inventory model.
* Documented how to install and use rpick on MacOS and Windows.


# 0.5.1

* [#32](https://gitlab.com/bowlofeggs/rpick/-/merge_requests/32): Fix an infinite loop when users
  say no to all possible choices when there are items in the list with no chance of being chosen.


# 0.5.0

* [#27](https://gitlab.com/bowlofeggs/rpick/merge_requests/27): Add a ```--config``` flag that
  allows users to specify a path to rpick's config file.


# 0.4.0

* [#20](https://gitlab.com/bowlofeggs/rpick/merge_requests/20): Add an LRU model.


# 0.3.1

* [#19](https://gitlab.com/bowlofeggs/rpick/merge_requests/19): Adjust the tests to pass on 32-bit
  architectures.


# 0.3.0

* [#5](https://gitlab.com/bowlofeggs/rpick/merge_requests/5): Introduced a unit test suite.
* [#8](https://gitlab.com/bowlofeggs/rpick/merge_requests/8): Defined a library for rpick so
  integrators can write their own front end to it. This also aided in testing.
* [#11](https://gitlab.com/bowlofeggs/rpick/merge_requests/11): Added documentation for the library.
* [#14](https://gitlab.com/bowlofeggs/rpick/merge_requests/14): Users will no longer be re-prompted
  for a choice they've declined in the same process, unless they decline all possible choices in a
  category.
* [d20e491b](https://gitlab.com/bowlofeggs/rpick/commit/d20e491b5971b73dd27d46bae3938f9321272517):
  Documented installation.


# 0.2.0

* [#3](https://gitlab.com/bowlofeggs/rpick/merge_requests/3): Added a new ```even``` distribution
  model, which does a nice flat random pick.
* [#4](https://gitlab.com/bowlofeggs/rpick/merge_requests/4): Added a new ```weighted```
  distribution model, which does a weighted random pick.
* [95b32b1e](https://gitlab.com/bowlofeggs/rpick/commit/95b32b1e4c103843cf3af900d94f5fef3ca286df):
  Added a new ```lottery``` distribution model, which gives lottery tickets to unpicked items and
  resets the picked item's lottery tickets to 0.


# 0.1.0

* [#1](https://gitlab.com/bowlofeggs/rpick/merge_requests/1): Added a new
  ```stddev_scaling_factor``` setting, which is optional and defaults to ```3.0```.
* [#2](https://gitlab.com/bowlofeggs/rpick/merge_requests/2): The model now defaults to "gaussian",
  so users don't have to define it by hand.
