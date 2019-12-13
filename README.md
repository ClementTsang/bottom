# bottom

[![Build Status](https://travis-ci.com/ClementTsang/bottom.svg?token=1wvzVgp94E1TZyPNs8JF&branch=master)](https://travis-ci.com/ClementTsang/bottom) [![crates.io link](https://img.shields.io/crates/v/bottom.svg)](https://crates.io/crates/bottom)

A top clone, written in Rust. Inspired by both [gtop](https://github.com/aksakalli/gtop) and [gotop](https://github.com/cjbassi/gotop)

![Quick demo recording](assets/recording_1.gif)

## Installation

### Linux

You can install by cloning and using `cargo build --release`, or download the pre-compiled binary in Releases.

### Windows

You can currently install by cloning and building yourself. You may need to install a font like [FreeMono](https://fonts2u.com/free-monospaced.font) and use a terminal like cmder for font support to work properly, unfortunately.

### MacOS

Currently, I'm unable to really dev or test on MacOS, so I'm not sure how well this will work, if at all. I'll try to source MacOS hardware to test this application.

## Usage

Note that all options and keybinds on GitHub may reflect the current development build, and not that of the current releases. For now, refer to the [crate](https://crates.io/crates/bottom) README for documentation as of time of release.

### Command line options

- `-h`, `--help` to show the help screen and exit (basically has all of the below CLI option info).

- `-a`, `--avgcpu` enables showing the average CPU usage on rustop.

- `-m`, `--dot-marker` uses a dot marker instead of the default braille marker. This is useful for things like Powershell which display braille markers incorrectly.

- `-c`, `--celsius` displays the temperature type in Celsius. This is the default.

- `-f`, `--fahrenheit` displays the temperature type in Fahrenheit. This is the default.

- `-k`, `--kelvin` displays the temperature type in Kelvin. This is the default.

- `-v`, `--version` displays the version number and exits.

- `-d`, `--debug` enables debug logging.

- `-r <RATE>`, `--rate <RATE>` will set the refresh rate in _milliseconds_. Pick a range from 250ms to `UINT_MAX`. Defaults to 1000ms, and higher values may take more resources due to more frequent polling of data, and may be less accurate in some circumstances.

### Keybinds

#### General

- `q`, `Ctrl-c` to quit.

- `Ctrl-r` to reset the screen and reset all collected data.

- `f` to freeze the screen from updating with new data. Press `f` again to unfreeze. Note that monitoring will still continue in the background.

- `Up/k`, `Down/j`, `Left/h`, `Right/l` to navigate between panels.

- `Shift+Up` and `Shift+Down` scrolls through the list if the panel is a table (Temperature, Disks, Processes).

- `Esc` to close a dialog window.

- `?` to get a help screen explaining the controls. Note all controls except `Esc` to close the dialog will be disabled while this is open.

#### Processes Panel

- `dd` to kill the selected process - **I would highly recommend you to be careful using this, lest you accidentally kill the wrong process**.

- `c` to sort by CPU usage. Sorts in descending order by default. Press again to reverse sorting order.

- `m` to sort by memory usage. Sorts in descending order by default. Press again to reverse sorting order.

- `p` to sort by PID. Sorts in ascending order by default. Press again to reverse sorting order.

- `n` to sort by process name. Sorts in ascending order by default. Press again to reverse sorting order.

### Mouse Actions

[* Scrolling either scrolls through the list if the panel is a table (Temperature, Disks, Processes), or zooms in and out if it is a chart.]: <>

- Scrolling currently only scrolls through the list if you are on the Processes panel. This will change in the future.

## Thanks

- As mentioned, this project is very much inspired by both [gotop](https://github.com/cjbassi/gotop) and [gtop](https://github.com/aksakalli/gtop) .

- This application was written with the following libraries:
  - [chrono](https://github.com/chronotope/chrono)
  - [clap](https://github.com/clap-rs/clap)
  - [crossterm](https://github.com/TimonPost/crossterm)
  - [failure](https://github.com/rust-lang-nursery/failure)
  - [fern](https://github.com/daboross/fern)
  - [futures-rs](https://github.com/rust-lang-nursery/futures-rs)
  - [futures-timer](https://github.com/rustasync/futures-timer)
  - [heim](https://github.com/heim-rs/heim)
  - [log](https://github.com/rust-lang-nursery/log)
  - [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
  - [tokio](https://github.com/tokio-rs/tokio)
  - [tui-rs](https://github.com/fdehau/tui-rs)
  - [winapi](https://github.com/retep998/winapi-rs)

## Why

I was looking to try writing more things in Rust, and I love the gotop tool. And thus, this project was born.
