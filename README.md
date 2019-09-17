# rustop

[![Build Status](https://travis-ci.com/ClementTsang/rustop.svg?token=1wvzVgp94E1TZyPNs8JF&branch=master)](https://travis-ci.com/ClementTsang/rustop)

A top clone, written in Rust.  Inspired by both [gtop](https://github.com/aksakalli/gtop) and [gotop](https://github.com/cjbassi/gotop)

![Quick demo recording](assets/recording_1.gif)
*Note that the background you see is not part of the app, that's just because I use a slightly transparent background*

## Installation

### Linux

TODO: Write

### Windows

TODO: Test

### MacOS

Currently, I'm unable to test on MacOS, so I'm not sure how well this will work, if at all.  I'll try to source MacOS hardware to test this application.

## Usage

### Keybinds (some may not be available on certain operating systems)

#### General

* ``q``, ``Esc``, or ``Ctrl-C`` to quit

* ``Shift-Up/Shift-k``, ``Shift-Down/Shift-j``, ``Shift-Left/Shift-h``, ``Shift-Right/Shift-l`` to navigate between panels

#### Processes Panel

* ``dd`` to kill the selected process (currently only on Linux) - **I would highly recommend you to be careful using this, lest you accidentally kill the wrong process**.

* ``c`` to sort by CPU usage.  Sorts in descending order by default.  Press again to reverse sorting order.

* ``m`` to sort by memory usage.  Sorts in descending order by default.  Press again to reverse sorting order.

* ``p`` to sort by PID.  Sorts in ascending order by default.  Press again to reverse sorting order.

* ``n`` to sort by process name.  Sorts in ascending order by default.  Press again to reverse sorting order.

### Mouse Actions

* Scrolling either scrolls through the list if the panel is a table (Temperature, Disks, Processes), or zooms in and out if it is a chart

## Thanks

* As mentioned, this project is very much inspired by both [gotop](https://github.com/cjbassi/gotop) and [gtop](https://github.com/aksakalli/gtop) .

* This application was written with the following libraries:
  * [clap](https://github.com/clap-rs/clap)
  * [crossterm](https://github.com/TimonPost/crossterm)
  * [heim](https://github.com/heim-rs/heim)
  * [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
  * [tokio](https://github.com/tokio-rs/tokio)
  * [tui-rs](https://github.com/fdehau/tui-rs) (note I used a fork due to some issues I faced, you can find that [here](https://github.com/ClementTsang/tui-rs))

## Why

I was looking to try writing more things in Rust, and I love the gotop tool.  And thus, this project was born.
