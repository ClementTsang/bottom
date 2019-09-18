# bottom

[![Build Status](https://travis-ci.com/ClementTsang/rustop.svg?token=1wvzVgp94E1TZyPNs8JF&branch=master)](https://travis-ci.com/ClementTsang/rustop) [![crates.io link](https://img.shields.io/crates/v/bottom.svg)](https://crates.io/crates/bottom)

A top clone, written in Rust.  Inspired by both [gtop](https://github.com/aksakalli/gtop) and [gotop](https://github.com/cjbassi/gotop)

![Quick demo recording](assets/recording_1.gif)
*Note that the background you see is not part of the app, that's just because I use a slightly transparent terminal.*

## Installation

### Linux

You can install by cloning and using ``cargo build --release``, or download the pre-compiled binary in Releases.  Note this needs the nightly toolchain if you are building.

### Windows

This is still in development, but will be done (hopefully) soon.

### MacOS

Currently, I'm unable to really dev or test on MacOS, so I'm not sure how well this will work, if at all.  I'll try to source MacOS hardware to test this application.

## Usage

### Command line options

* ``-h``, ``--help`` to show the help screen and exit (basically has all of the below CLI option info).

* ``-a``, ``--avgcpu`` enables showing the average CPU usage on rustop

* ``-c``, ``--celsius`` displays the temperature type in Celsius.  This is the default.

* ``-f``, ``--fahrenheit`` displays the temperature type in Fahrenheit.  This is the default.

* ``-k``, ``--kelvin`` displays the temperature type in Kelvin.  This is the default.

* ``-v``, ``--version`` displays the version number and exits.

* ``-r <RATE>``, ``--rate <RATE>`` will set the refresh rate in *milliseconds*.  Pick a range from 250ms to ``UINT_MAX``.  Defaults to 1000ms, and higher values may take more resources due to more frequent polling of data, and may be less accurate in some circumstances.

### Keybinds (some may not be available on certain operating systems)

#### General

* ``q``, ``Esc``, or ``Ctrl-C`` to quit.

* ``Up/k``, ``Down/j``, ``Left/h``, ``Right/l`` to navigate between panels.  This currently doesn't have much functionality but will change in the future.

#### Processes Panel

* ``dd`` to kill the selected process (currently only on Linux) - **I would highly recommend you to be careful using this, lest you accidentally kill the wrong process**.

* ``c`` to sort by CPU usage.  Sorts in descending order by default.  Press again to reverse sorting order.

* ``m`` to sort by memory usage.  Sorts in descending order by default.  Press again to reverse sorting order.

* ``p`` to sort by PID.  Sorts in ascending order by default.  Press again to reverse sorting order.

* ``n`` to sort by process name.  Sorts in ascending order by default.  Press again to reverse sorting order.

### Mouse Actions

[* Scrolling either scrolls through the list if the panel is a table (Temperature, Disks, Processes), or zooms in and out if it is a chart.]: <>

* Scrolling currently only scrolls through the list if you are on the Processes panel.  This will change in the future.

## Regarding Process Use Percentage (on Linux)

I cannot guarantee whether they are 100% accurate.  I'm using a technique I found online which seems to be a better indicator of process use percentage at the current time, rather than pulling from, say, ``ps`` (which is average CPU usage over the *entire lifespan* of the process).  I found the options from the library I used to get other data (heim) to be a bit too inaccurate in my testing.

For reference, the formula I am using to calculate CPU process usage is along the lines of:

```rust
let idle = idle + iowait;
let non_idle = user + nice + system + irq + softirq + steal + guest;

let total = idle + non_idle;
let prev_total = prev_idle + prev_non_idle; // Both of these values are calculated using the same formula from the previous polling

let total_delta : f64 = total - prev_total;
let idle_delta : f64 = idle - prev_idle;

let final_delta : f64 = total_delta - idle_delta;

//...

// Get utime and stime from reading /proc/<PID>/stat
let after_proc_val = utime + stime;
(after_proc_val - before_proc_val) / cpu_usage * 100_f64; //This gives your use percentage.  before_proc_val comes from the previous polling
```

Any suggestions on more accurate data is welcome!  Note all other fields should be accurate.

## Thanks

* As mentioned, this project is very much inspired by both [gotop](https://github.com/cjbassi/gotop) and [gtop](https://github.com/aksakalli/gtop) .

* This application was written with the following libraries:
  * [chrono](https://github.com/chronotope/chrono)
  * [clap](https://github.com/clap-rs/clap)
  * [crossterm](https://github.com/TimonPost/crossterm)
  * [failure](https://github.com/rust-lang-nursery/failure)
  * [fern](https://github.com/daboross/fern)
  * [futures-rs](https://github.com/rust-lang-nursery/futures-rs)
  * [futures-timer](https://github.com/rustasync/futures-timer)
  * [heim](https://github.com/heim-rs/heim)
  * [log](https://github.com/rust-lang-nursery/log)
  * [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
  * [tokio](https://github.com/tokio-rs/tokio)
  * [tui-rs](https://github.com/fdehau/tui-rs) (note I used a fork due to some issues I faced, you can find that [here](https://github.com/ClementTsang/tui-rs))

## Why

I was looking to try writing more things in Rust, and I love the gotop tool.  And thus, this project was born.

