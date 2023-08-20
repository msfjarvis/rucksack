# rucksack [![No Maintenance Intended](http://unmaintained.tech/badge.svg)](http://unmaintained.tech/) [![Built with Garnix](https://img.shields.io/static/v1?label=Built%20with&message=Garnix&color=blue&style=flat&logo=nixos&link=https://garnix.io&labelColor=111212)](https://garnix.io)

rucksack is a simple file moving service that was built to solve the use case of watching a collection of directories and collecting all there files into a single target directory.

The idea for this was born from the frustration of playing games and wanting to share screenshots from them with my friends. Every game likes to hide its screenshots in a specific obscure path which made it harder to find them when I needed to. With rucksack they can all neatly stay in a single folder making discoverability significantly easier.

## Usage

### Configuration

An example config file can look something like this:

```toml
# ~/.config/rucksack.toml
name = "Screenshots" # Optional
sources = [
  "/mnt/Games/Minecraft/screenshots",
  "/c/Users/Harsh Shandilya/Pictures/God Of War"
]
target = "/mnt/mediahell/screenshots"
file_filter = "*.png"
```

### Running

Prebuilt binaries for macOS and Linux can be installed from [here](https://github.com/msfjarvis/rucksack/releases/latest).

`rucksack` uses [watchman](https://github.com/facebook/watchman) to power its file-watching capabilities. You can find the steps to install it for your own platform [here](https://facebook.github.io/watchman/docs/install).

To build from source, clone this repository and run `cargo run --release`. You will require a Rust installation.

`rucksack` is only tested against the latest stable release of Rust but a few versions older should also be fine.

## Licensing

Dual licensed under Apache 2.0 or MIT at your option.
