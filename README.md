```rpick``` is a command line tool that helps you to pick items from a sorted list, using a Gaussian
probability model. Things near the top of the list have the highest probability of being chosen,
while things at the end of the list have the lowest chance. Once an item has been picked and the
user has accepted the choice, the list is saved to disk with the picked item moved to the end of
the list.

An example use case for this is picking a restaurant. You might want to generally go to restaurants
you haven't visited in a while, but you also might not want to use a strict least recently used
model and spice things up with some element of chance, with restaurants you've least recently
visited getting a boost in their chances.

```rpick``` keeps its lists in a [YAML](https://yaml.org/) file in your home config directory called
```rpick.yml```. For now, users must create this file by hand, and ```rpick``` will manage it from
there. To get started with our restaurant example, create ```~/.config/rpick.yml``` like this:

```
---
restaurant:
  choices:
    - Spirits
    - Lucky 32
    - Centro
    - Sitti
    - Cookout
```

Then you can ask ```rpick``` to pick one for you:

```
$ rpick restaurant
Choice is Lucky 32. Accept? (Y/n)
```

If you say yes, it will rewrite the yaml file like this:

```
---
restaurant:
  model: gaussian
  stddev_scaling_factor: 3.0
  choices:
    - Spirits
    - Centro
    - Sitti
    - Cookout
    - Lucky 32
```

Note that we passed ```restaurant``` as an argument to ```rpick``` - this told ```rpick``` to look
for the ```restaurant``` object in ```rpick.yml``` to find out which model to use and which choices
were available. This parameter is required, but its possible values are defined by you in your
config file.

Note that it added two settings that weren't there originally, ```model```, and
```stddev_scaling_factor```.

The ```model``` field in the config file defines which mathematical
model to use to pick from the given choices, and at the time of writing only ```gaussian```
is a valid model. There are tentative plans to add other models later in the future.

```stddev_scaling_factor``` is used to derive the standard deviation; the standard deviation is the
length of the list of choices, divided by this scaling factor. Thus, a larger scaling factor will
result in a stronger preference for items near the top of the list, and a smaller scaling factor
will result in a more even distribution among the choices. Note that the smaller the scaling factor
is, the longer rpick will take to make a decision, on average. The default is ```3.0```, which is
chosen because it places the last item on the list at three standard deviations, giving it a 0.03%
chance of being chosen.

This project is available on [crates.io](https://crates.io/crates/rpick).


# Changelog

## 0.1.0

* [#1](https://gitlab.com/bowlofeggs/rpick/merge_requests/1): Added a new
  ```stddev_scaling_factor``` setting, which is optional and defaults to ```3.0```.
* [#2](https://gitlab.com/bowlofeggs/rpick/merge_requests/2): The model now defaults to "gaussian",
  so users don't have to define it by hand.
