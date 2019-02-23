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
  model: gaussian
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

The ```model``` field in the config file is required, but at the time of writing only ```gaussian```
is a valid model. There are tentative plans to add other models later in the future.

This project is available on [crates.io](https://crates.io/crates/rpick).
