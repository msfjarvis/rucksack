# file-collector [![No Maintenance Intended](http://unmaintained.tech/badge.svg)](http://unmaintained.tech/) [![Built with Garnix](https://img.shields.io/static/v1?label=Built%20with&message=Garnix&color=blue&style=flat&logo=nixos&link=https://garnix.io&labelColor=111212)](https://garnix.io)

I often run into an annoying problem where every game I play stores in-game screenshots into a different hard-to-access directory and it becomes a pain to hunt them down when I want to share one of them.

This simple tool aims to resolve that using the concept of a bucket, where you define a number of source directories from where files should be picked up from and a target directory where they should be moved to.

## Usage

### Configuration

An example config file can look something like this:

```toml
# ~/.config/collector/config.toml
[bucket]
name = "Screenshots"
sources = [
  "/mnt/mediahell/MultiMC/instances/Fabulously-Optimized-4.1.0-beta.2/minecraft/screenshots",
  "/mnt/mediahell/MultiMC/instances/Fabulously Optimized 4.2.0-beta.1/.minecraft/screenshots"
]
target = "/mnt/mediahell/screenshots"
```

### Running

Clone this repository and run `cargo run --release`. You will require a Rust installation.

## Licensing

Dual licensed under Apache 2.0 or MIT at your option.
