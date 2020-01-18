# bottom

[![Build Status](https://travis-ci.com/ClementTsang/bottom.svg?token=1wvzVgp94E1TZyPNs8JF&branch=master)](https://travis-ci.com/ClementTsang/bottom) [![crates.io link](https://img.shields.io/crates/v/bottom.svg)](https://crates.io/crates/bottom)

A graphical top clone, written in Rust. Inspired by both [gtop](https://github.com/aksakalli/gtop) and [gotop](https://github.com/cjbassi/gotop)

![Quick demo recording](assets/recording_1.gif)

## Features

Features of bottom include:

- CPU widget to show a visual representation of per-core usage. Average CPU display also exists.

- Memory widget to show a visual representation of both RAM and SWAP usage.

- Networks widget to show a log-based visual representation of network usage.

- Sortable and searchable process widget. Searching supports regex, and you can search by PID and process name.

- Disks widget to display usage and I/O per second.

- Temperature widget to monitor detected sensors in your system.

The compatibility of each widget and operating systems are, as of version 0.1.0, as follows:

| OS/Widget | CPU      | Memory   | Disks    | Temperature           | Processes | Networks                                      |
| --------- | -------- | -------- | -------- | --------------------- | --------- | --------------------------------------------- |
| Linux     | ✓        | ✓        | ✓        | ✓                     | ✓         | ✓                                             |
| Windows   | ✓        | ✓        | ✓        | Currently not working | ✓         | Partially supported (total RX/TX unavailable) |
| macOS     | Untested | Untested | Untested | Untested              | Untested  | Untested                                      |

## Installation

### Linux

You can install the in-development version by cloning and using `cargo build --release`. You can get release versions using `cargo install bottom`, or manually building from the Releases page. Other installation methods based on distros are as follows:

#### Arch Linux

You can get the release versions from the AUR by installing `bottom`.

### Windows

You can install the in-development version by cloning and using `cargo build --release`. You can get release versions using `cargo install bottom`, or manually building from the Releases page. You may need to install a font like [FreeMono](https://fonts2u.com/free-monospaced.font) and use a terminal like cmder for font support to work properly, unfortunately.

### macOS

macOS support will hopefully come soon<sup>TM</sup>.

## Usage

Run using `btm`.

### Command line options

- `-h`, `--help` shows the help screen and exits.

- `-a`, `--avgcpu` enables also showing the average CPU usage in addition to per-core CPU usage.

- `-m`, `--dot-marker` uses a dot marker instead of the default braille marker.

- `-c`, `--celsius` displays the temperature type in Celsius. This is the default.

- `-f`, `--fahrenheit` displays the temperature type in Fahrenheit.

- `-k`, `--kelvin` displays the temperature type in Kelvin.

- `-v`, `--version` displays the version number and exits.

- `-d`, `--debug` enables debug logging.

- `-r <RATE>`, `--rate <RATE>` will set the refresh rate in _milliseconds_. Lowest it can go is 250ms, the highest it can go is 2<sup>128</sup> - 1. Defaults to 1000ms, and lower values may take more resources due to more frequent polling of data, and may be less accurate in some circumstances.

- `-l`, `--left_legend` will move external table legends to the left side rather than the right side. Right side is default.

- `-u`, `--current_usage` will make a process' CPU usage be based on the current total CPU usage, rather than assuming 100% CPU usage. Only affects Linux for now.

- `-g`, `--group` will group together processes with the same name by default (equivalent to pressing `Tab`).

- `-s`, `--simple_search` will default the search to simple search rather than regex search.

### Keybindings

#### General

- `q`, `Ctrl-c` to quit. Note if you are currently in the search widget, `q` will not work so you can still type.

- `Ctrl-r` to reset the screen and reset all collected data.

- `f` to freeze the screen from updating with new data. Press `f` again to unfreeze. Note that monitoring will still continue in the background.

- `Ctrl-Up` or `Ctrl-k`, `Ctrl-Down` or `Ctrl-j`, `Ctrl-Left` or `Ctrl-h`, `Ctrl-Right` or `Ctrl-l` to navigate between widgets.

- `Esc` to close a dialog window.

- `?` to get a help screen explaining the controls. Note all controls except `Esc` to close the dialog will be disabled while this is open.

#### Scrollable Tables

- `Up` or `k` and `Down` or `j` scrolls through the list if the widget is a table (Temperature, Disks, Processes).

- `gg` or `Home` to jump to the first entry of the current table.

- `G` (`Shift-g`) or `End` to jump to the last entry of the current table.

#### Processes

- `dd` to kill the selected process

- `c` to sort by CPU usage. Sorts in descending order by default. Press again to reverse sorting order.

- `m` to sort by memory usage. Sorts in descending order by default. Press again to reverse sorting order.

- `p` to sort by PID. Sorts in ascending order by default. Press again to reverse sorting order.

- `n` to sort by process name. Sorts in ascending order by default. Press again to reverse sorting order.

- `Tab` to group together processes with the same name. Disables PID sorting. `dd` will now kill all processes covered by that name.

- `Ctrl-f` or `/` to open the search widget.

#### Search Widget

- `Ctrl-p` or `Ctrl-n` to switch between searching for PID and name respectively.

- `Ctrl-s` to toggle between a simple search and a regex search.

- `Ctrl-a` and `Ctrl-e` to jump to the start and end of the search bar respectively.

- `Esc` or `Ctrl-f` to close.

Note that `q` is disabled while in the search widget.

### Mouse actions

- Scrolling with the mouse will scroll through the currently selected list.

## Thanks, kudos, and all the like

- This project is very much inspired by both [gotop](https://github.com/cjbassi/gotop) and [gtop](https://github.com/aksakalli/gtop) .

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
  - [lazy_static](https://github.com/rust-lang-nursery/lazy-static.rs)
