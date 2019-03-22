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
