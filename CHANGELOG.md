# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - Unreleased

### Features

- [#58](https://github.com/ClementTsang/bottom/issues/58): I/O stats per process.

- [#55](https://github.com/ClementTsang/bottom/issues/55): Battery monitoring widget.

- [#114](https://github.com/ClementTsang/bottom/pull/114): Process state per process.

### Changes

- Changed default colours for highlighted borders and table headers to light blue - this is mostly to deal with Powershell colour conflicts.

- Updated the widget type keyword list to accept the following keywords as existing types:

  - `"memory"`
  - `"network"`
  - `"process"`
  - `"processes"`
  - `"temperature"`

- [#117](https://github.com/ClementTsang/bottom/issues/117): Update tui to 0.9:

  - Removed an (undocumented) feature in allowing modifying total RX/TX colours. This is mainly due to the legend change.

  - Use custom legend-hiding to stop hiding legends for memory and network widgets.

  - In addition, changed to using only legends within the graph for network, as well as redesigned the legend.
    The old legend style can still be used via the `--use_old_network_legend` flag or `use_old_network_legend = true` config option.

  - Allow for option to hide the header gap on tables via `--hide_table_gap` or `hide_table_gap = true`.

- [#126](https://github.com/ClementTsang/bottom/pull/126): Updated error messages to be a bit more consistent/helpful.

- [#70](https://github.com/ClementTsang/bottom/issues/70): Redesigned help menu to allow for scrolling.

- [#134](https://github.com/ClementTsang/bottom/pull/134): Added `hjkl` movement to delete dialog.

- [#59](https://github.com/ClementTsang/bottom/issues/59): Redesigned search menu and query.

### Bug Fixes

- Fixed `dd` not working on non-first entries.

- Fixed bug where a single empty row as a layout would crash without a proper warning.
  The behaviour now errors out with a more helpful message.

- Fixed bug where empty widgets in layout would cause widget movement to not work properly when moving vertically.

### Development changes

- Switch to stateful widget style for tables.

- [#38](https://github.com/ClementTsang/bottom/issues/38): Updated arg tests and added config testing.

- More refactoring.

## [0.3.0] - 2020-04-07

### Features

- [#20](https://github.com/ClementTsang/bottom/issues/20): Time scaling was added to allow users to zoom in/out based on their desired time intervals. Time markers on the charts can be hidden or automatically hidden.

- [#37](https://github.com/ClementTsang/bottom/issues/37): Automatically populate a config file if one does not exist.

- [#21](https://github.com/ClementTsang/bottom/issues/21): Basic mode added.

- [#51](https://github.com/ClementTsang/bottom/issues/51): Modularity with widget placement or inclusion added.

### Changes

- Removed redundant dependencies.

- [#17](https://github.com/ClementTsang/bottom/issues/17): Add colouring options to the total RX/TX labels.

- [#29](https://github.com/ClementTsang/bottom/issues/29): Added `F1-F3` keys as alternatives for selecting search options

- [#42](https://github.com/ClementTsang/bottom/issues/42), [#45](https://github.com/ClementTsang/bottom/issues/45), [#35](https://github.com/ClementTsang/bottom/issues/35): Change the arrow used for sorting processes to work with other terminals.

- [#61](https://github.com/ClementTsang/bottom/issues/61): Search box changed to not block if the window is small.

- [#40](https://github.com/ClementTsang/bottom/issues/40): Rewrote README to be more clear and explicit.

- [#109](https://github.com/ClementTsang/bottom/issues/109): Sorting processes by name is case-insensitive.

### Bug Fixes

- [#33](https://github.com/ClementTsang/bottom/issues/33): Fix bug with search and graphemes bigger than a byte crashing due to the cursor.

- [#41](https://github.com/ClementTsang/bottom/issues/41): Fix bug that caused the cursor to go off screen while searching.

- [#61](https://github.com/ClementTsang/bottom/issues/61): Dialog boxes set to be a constant width/height.

- [#80](https://github.com/ClementTsang/bottom/issues/80): Fix bug with resizing and scrolling causing issues with tables.

- [#77](https://github.com/ClementTsang/bottom/issues/77): Fixed hidden CPU entries from being scrolled to.

- [#79](https://github.com/ClementTsang/bottom/issues/79): Fixed CPU entries being a different colour if the one above it was hidden.

- [#85](https://github.com/ClementTsang/bottom/pull/85): A div-by-zero error when the memory values were zero was fixed.

### Other

- Various Travis changes.

- Scoop install option added.

## [0.2.2] - 2020-02-26

### Features

- Added support for colouring the average CPU core separately in config files.

- [#15](https://github.com/ClementTsang/bottom/issues/15) - Added support for (some) named colours and RGB values in config files.

### Bug Fixes

- [#28](https://github.com/ClementTsang/bottom/issues/30): Fixed broken Cargo.toml for Cargo installs.

- Fixed Windows issue with shift key.

- [#14](https://github.com/ClementTsang/bottom/issues/14): Ignore certain characters in search

## [0.2.1] - 2020-02-21

### Bug Fixes

- [#14](https://github.com/ClementTsang/bottom/issues/11): Fixed default config paths not being read properly.

## [0.2.0] - 2020-02-20

### Features

- Searching in processes was added.

- The option of a config file was added. Config files follow the TOML spec. These support boot flags by default, and colour schemes.

- The capability of maximizing a widget to take up all draw space was added.

- Filtering out CPU cores on the graph/legend was added.

### Changes

- Default colours were changed for better support on macOS Terminal and PowerShell.

- Rewrote and refactored how I get data to be less spaghetti. This might also have the added benefit of running better, with less duplicated logic.

- Changed how the dd dialog and help dialog look. Hopefully they'll be nicer to look at and more intuitive to use!

### Bug Fixes

- [#2](https://github.com/ClementTsang/bottom/issues/2): Fixed issues where the program would crash if the window was too small.

- Added a panic handler so terminals won't get all broken if a panic _does_ still occur.

- Fixed some sizing issues, hopefully this means that it's still readable at smaller sizes (within reason).

- [#10](https://github.com/ClementTsang/bottom/issues/10): Fixed scroll issue caused by resizing.

## [0.1.2] - 2020-01-11

### Changes

- Added a bit more complexity to how we determine column widths for tables. This should fix an issue where columns would glitch out at smaller widths, and hopefully look nicer.

### Bug Fixes

- Rewrote scroll logic in tables to avoid some strange scroll behaviour I encountered where it would jump around.

- Attempt to patch a panic caused by the change in how we determine time in the data collection stage.

## [0.1.1] - 2020-01-11

### Features

- `Tab` in the processes widget will now group similarly-named processes together (as well as their total CPU and MEM usage). `dd`-ing this will try to kill all entries with that process name.

- A flag to enable this by default is also now available.

### Bug Fixes

- Accidentally left in a bug in which the disk widget was using megabytes instead of bytes as their unit during data collection... but during data conversion for the display I treated them as bytes.

## [0.1.0] - 2020-01-11

Initial release.
