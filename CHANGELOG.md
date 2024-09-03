# Changelog

All notable changes to this project will be documented in this file. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Versioning for this project is based on [Semantic Versioning](https://semver.org/spec/v2.0.0.html). More specifically:

**Pre 1.0.0 (current)**:

- Patch versions should aim to only contain bug fixes or non-breaking features/changes.
- Minor versions may break things.

**Post 1.0.0**:

- Patch versions should only contain bug fixes.
- Minor versions should only contain forward-compatible features/changes.
- Major versions may break things.

That said, these are more guidelines rather than hardset rules, though the project will generally try to follow them.

---

## [0.11.0]/[0.10.3] - Unreleased

### Bug Fixes

- [#1551](https://github.com/ClementTsang/bottom/pull/1551): Fix missing parent section names in default config.
- [#1552](https://github.com/ClementTsang/bottom/pull/1552): Fix typo in default config.
- [#1578](https://github.com/ClementTsang/bottom/pull/1578): Fix missing selected text background colour in `default-light` theme.

### Changes

- [#1559](https://github.com/ClementTsang/bottom/pull/1559): Rename `--enable_gpu` to `--disable_gpu`, and make GPU features enabled by default.
- [#1570](https://github.com/ClementTsang/bottom/pull/1570): Consider `$XDG_CONFIG_HOME` on macOS when looking for a default config path in a
  backwards-compatible fashion.

## [0.10.2] - 2024-08-05

### Features

- [#1487](https://github.com/ClementTsang/bottom/pull/1487): Add option to move the AVG CPU bar to another row in basic mode.

### Bug Fixes

- [#1541](https://github.com/ClementTsang/bottom/pull/1541): Fix some process details not updating for macOS and Windows.
- [#1542](https://github.com/ClementTsang/bottom/pull/1542): Fix confusing process run times being reported on macOS.
- [#1543](https://github.com/ClementTsang/bottom/pull/1543): Fix the `--default_cpu_entry` argument not being checked.

## [0.10.1] - 2024-08-01

### Bug Fixes

- [#1526](https://github.com/ClementTsang/bottom/pull/1526): Fix `--help` description being incorrectly set for a flag, breaking the output.

## [0.10.0] - 2024-08-01

### Features

- [#1276](https://github.com/ClementTsang/bottom/pull/1276): Add GPU process info.
- [#1353](https://github.com/ClementTsang/bottom/pull/1353): Support selecting the average CPU graph as a default.
- [#1373](https://github.com/ClementTsang/bottom/pull/1373): Add support for bcachefs in disk widget.
- [#1430](https://github.com/ClementTsang/bottom/pull/1430): Support controlling the graph legend position for memory and network graph widgets.
- [#1512](https://github.com/ClementTsang/bottom/pull/1512): Support bold text styling options.
- [#1514](https://github.com/ClementTsang/bottom/pull/1514): Support italic text styling options.

### Changes

- [#1276](https://github.com/ClementTsang/bottom/pull/1276): NVIDIA GPU functionality is now tied behind the `--enable_gpu` flag. This will likely be changed in the future.
- [#1344](https://github.com/ClementTsang/bottom/pull/1344): Change the `group` command line-argument to `group_processes` for consistency with the config file option.
- [#1376](https://github.com/ClementTsang/bottom/pull/1376): Group together related command-line arguments in `-h` and `--help`.
- [#1411](https://github.com/ClementTsang/bottom/pull/1411): Add `time` as a default column.
- [#1436](https://github.com/ClementTsang/bottom/pull/1436): Use actual "swap" value for Windows.
- [#1441](https://github.com/ClementTsang/bottom/pull/1441): The following arguments have changed names:
  - `--left_legend/-l` is now `--cpu_left_legend`.
- [#1441](https://github.com/ClementTsang/bottom/pull/1441): The following config fields have changed names:
  - `expanded_on_startup` is now `expanded`.
  - `left_legend` is now `cpu_left_legend`.
- [#1458](https://github.com/ClementTsang/bottom/pull/1458): Fix a bug with `--hide_table_gap` with the battery widget.
- [#1472](https://github.com/ClementTsang/bottom/pull/1472): The following arguments have changed names:
  - `--mem_as_value` is now `process_memory_as_value`.
- [#1472](https://github.com/ClementTsang/bottom/pull/1472): The following config fields have changed names:
  - `mem_as_value` is now `process_memory_as_value`.
- [#1481](https://github.com/ClementTsang/bottom/pull/1481): The following config fields have changed names:
  - `disk_filter` is now `disk.name_filter`.
  - `mount_filter` is now `disk.mount_filter`.
  - `temp_filter` is now `temperature.sensor_filter`
  - `net_filter` is now `network.interface_filter`
- [#1499](https://github.com/ClementTsang/bottom/pull/1499): Redesign how styling is configured.
- [#1499](https://github.com/ClementTsang/bottom/pull/1499): The following arguments have changed names:
  - `--colors` is now `--theme`
- [#1513](https://github.com/ClementTsang/bottom/pull/1513): Table headers are now bold by default.
- [#1515](https://github.com/ClementTsang/bottom/pull/1515): Show the config path in the error message if unable to read/create a config.

### Bug Fixes

- [#1314](https://github.com/ClementTsang/bottom/pull/1314): Fix fat32 mounts not showing up in macOS.
- [#1355](https://github.com/ClementTsang/bottom/pull/1355): Reduce chances of non-D0 devices waking up due to temperature checks on Linux.
- [#1410](https://github.com/ClementTsang/bottom/pull/1410): Fix uptime calculation for Linux.

### Other

- [#1394](https://github.com/ClementTsang/bottom/pull/1394): Add JSON Schema support.

## [0.9.7] - 2024-07-26

## Bug Fixes

- [#1500](https://github.com/ClementTsang/bottom/issues/1500): Fix builds for Rust 1.80.

## [0.9.6] - 2023-08-26

### Other

- [#1286](https://github.com/ClementTsang/bottom/pull/1286): Pin serde to 1.0.188 to help with potential `cargo install` issues. Note this version should be fine and not pull in binaries.

## [0.9.5] - 2023-08-26

### Other

- [#1278](https://github.com/ClementTsang/bottom/pull/1278): Pin serde to 1.0.171.

## [0.9.4] - 2023-08-05

### Features

- [#1248](https://github.com/ClementTsang/bottom/pull/1248): Add I/O counters from ZFS for Linux and FreeBSD.

### Changes

- [#1236](https://github.com/ClementTsang/bottom/pull/1236): Hide the battery tab selector if there is only one battery detected.
- [#1251](https://github.com/ClementTsang/bottom/pull/1251): Make the charge meter take the entire width of the battery widget.

### Bug Fixes

- [#1230](https://github.com/ClementTsang/bottom/pull/1230): Fix core dump if the terminal is closed while bottom is open.
- [#1245](https://github.com/ClementTsang/bottom/pull/1245): Fix killing processes in Windows leaving a handle open.
- [#1264](https://github.com/ClementTsang/bottom/pull/1264): Fix ARC usage showing max system memory instead of max ARC size.

## [0.9.3] - 2023-06-25

### Features

- [#1221](https://github.com/ClementTsang/bottom/pull/1221): Support human times for `rate`.

### Bug Fixes

- [#1216](https://github.com/ClementTsang/bottom/pull/1216): Fix arguments not being sorted alphabetically.
- [#1219](https://github.com/ClementTsang/bottom/pull/1219): Fix overflow/underflow in graph timespan zoom.

### Other

- [#1206](https://github.com/ClementTsang/bottom/pull/1206): Add `.rpm` package generation.
- [#1220](https://github.com/ClementTsang/bottom/pull/1220): Update documentation for features supporting human times.

## [0.9.2] - 2023-06-11

### Features

- [#1172](https://github.com/ClementTsang/bottom/pull/1172): Support human times for `time_delta` and `default_time_value`.
- [#1187](https://github.com/ClementTsang/bottom/pull/1187): Use better names for duplicate temp sensors found by `/sys/class/thermal`.
- [#1188](https://github.com/ClementTsang/bottom/pull/1188): Also check `/sys/devices/platform/coretemp.*` for temp sensors.

### Bug Fixes

- [#1186](https://github.com/ClementTsang/bottom/pull/1186): Fix for temperature sensor data gathering on Linux immediately halting if any method failed.
- [#1191](https://github.com/ClementTsang/bottom/pull/1191): Fix ntfs3 mounts not being counted as a physical drive type.
- [#1195](https://github.com/ClementTsang/bottom/pull/1195): Fix battery health being incorrectly reported on M1 macOS.
- [#1188](https://github.com/ClementTsang/bottom/pull/1188): Don't fail fast with temperature sensor name generation on Linux.

### Other

- [#1199](https://github.com/ClementTsang/bottom/pull/1199): bottom should build on `aarch64-linux-android` with features disabled.

## [0.9.1] - 2023-05-14

### Bug Fixes

- [#1148](https://github.com/ClementTsang/bottom/pull/1148): Fix Gruvbox colour string being invalid when cache usage is enabled.

## [0.9.0] - 2023-05-10

### Features

- [#1016](https://github.com/ClementTsang/bottom/pull/1016): Add support for displaying process usernames on Windows.
- [#1022](https://github.com/ClementTsang/bottom/pull/1022): Support three-character hex colour strings for styling.
- [#1024](https://github.com/ClementTsang/bottom/pull/1024): Support FreeBSD temperature sensors based on `hw.temperature`.
- [#1063](https://github.com/ClementTsang/bottom/pull/1063): Add buffer and cache memory tracking.
- [#1106](https://github.com/ClementTsang/bottom/pull/1106): Add current battery charging state.
- [#1115](https://github.com/ClementTsang/bottom/pull/1115): Add customizable process columns to config file.
- [#801](https://github.com/ClementTsang/bottom/pull/801): Add optional process time column and querying.

### Changes

- [#1025](https://github.com/ClementTsang/bottom/pull/1025): Officially support M1 macOS.
- [#1035](https://github.com/ClementTsang/bottom/pull/1035): Migrate away from heim for CPU information.
- [#1036](https://github.com/ClementTsang/bottom/pull/1036): Migrate away from heim for memory information; bottom will now try to use `MemAvailable` on Linux to determine used memory.
- [#1041](https://github.com/ClementTsang/bottom/pull/1041): Migrate away from heim for network information.
- [#1064](https://github.com/ClementTsang/bottom/pull/1064): Migrate away from heim for storage information.
- [#812](https://github.com/ClementTsang/bottom/issues/812): Fully remove heim from bottom.
- [#1075](https://github.com/ClementTsang/bottom/issues/1075): Update how drives are named in Windows.
- [#1106](https://github.com/ClementTsang/bottom/pull/1106): Rename battery consumption field to rate.

### Bug Fixes

- [#1021](https://github.com/ClementTsang/bottom/pull/1021): Fix selected text background colour being wrong if only the foreground colour was set.
- [#1037](https://github.com/ClementTsang/bottom/pull/1037): Fix `is_list_ignored` accepting all results if set to `false`.
- [#1064](https://github.com/ClementTsang/bottom/pull/1064): Disk name/mount filter now doesn't always show all entries if one filter wasn't set.
- [#1064](https://github.com/ClementTsang/bottom/pull/1064): macOS disk I/O is potentially working now.
- [#597](https://github.com/ClementTsang/bottom/issues/597): Resolve RUSTSEC-2021-0119 by removing heim.

### Other

- [#1100](https://github.com/ClementTsang/bottom/pull/1100): Speed up first draw and first data collection.
- [#1107](https://github.com/ClementTsang/bottom/pull/1107): Update to clap v4.
- [#1111](https://github.com/ClementTsang/bottom/pull/1111): Update to regex [1.8.0](https://github.com/rust-lang/regex/blob/93316a3b1adc43cc12fab6c73a59f646658cd984/CHANGELOG.md#180-2023-04-20), supporting more escapable characters and named captures.

## [0.8.0] - 2023-01-22

### Features

- [#950](https://github.com/ClementTsang/bottom/pull/950): Split usage into both usage percentage and usage value.

### Changes

- [#974](https://github.com/ClementTsang/bottom/pull/974): Hide battery duration section if the value is unknown. Also update shortened text.
- [#975](https://github.com/ClementTsang/bottom/pull/975): Automatically hide the battery widget if no batteries are found but `--battery` is enabled.

### Bug Fixes

- [#950](https://github.com/ClementTsang/bottom/pull/950): Update help menu for disk and temperature widgets with sorting support.
- [#994](https://github.com/ClementTsang/bottom/pull/994): Fix time graph labels not being styled.

### Other

- [#969](https://github.com/ClementTsang/bottom/pull/969): Follow Debian conventions for naming generated `.deb` binaries.

## [0.7.1] - 2023-01-06

### Bug Fixes

- [#950](https://github.com/ClementTsang/bottom/pull/950): Fix invalid sorting order for disk usage percentage.
- [#952](https://github.com/ClementTsang/bottom/pull/952), [#960](https://github.com/ClementTsang/bottom/pull/960): Partially fix battery text getting cut off in small windows.
- [#953](https://github.com/ClementTsang/bottom/pull/953): Fix CPU widget's 'all' label being missing on small sizes.

### Other

- [#951](https://github.com/ClementTsang/bottom/pull/951): Nightly builds now have their version number (`btm -V`) tagged with the commit hash.

## [0.7.0] - 2022-12-31

### Features

- [#676](https://github.com/ClementTsang/bottom/pull/676): Add support for NVIDIA GPU temperature sensors.
- [#760](https://github.com/ClementTsang/bottom/pull/760): Add a check for whether bottom is being run in a terminal.
- [#766](https://github.com/ClementTsang/bottom/pull/766): Add FreeBSD support.
- [#774](https://github.com/ClementTsang/bottom/pull/774): Add half page scrolling with `ctrl-u` and `ctrl-d`.
- [#784](https://github.com/ClementTsang/bottom/pull/784): Add ZFS ARC support.
- [#794](https://github.com/ClementTsang/bottom/pull/794): Add GPU memory support for NVIDIA GPUs.
- [#806](https://github.com/ClementTsang/bottom/pull/806): Update sysinfo to support M1 macOS temperature sensors.
- [#836](https://github.com/ClementTsang/bottom/pull/836): Add CLI options for GPU memory.
- [#841](https://github.com/ClementTsang/bottom/pull/841): Add page up/page down support for the help screen.
- [#868](https://github.com/ClementTsang/bottom/pull/868): Make temperature widget sortable.
- [#870](https://github.com/ClementTsang/bottom/pull/870): Make disk widget sortable.
- [#881](https://github.com/ClementTsang/bottom/pull/881): Add pasting to the search bar.
- [#892](https://github.com/ClementTsang/bottom/pull/892): Add custom retention periods for data.
- [#899](https://github.com/ClementTsang/bottom/pull/899), [#910](https://github.com/ClementTsang/bottom/pull/910), [#912](https://github.com/ClementTsang/bottom/pull/912): Add non-normalized CPU usage to processes.
- [#919](https://github.com/ClementTsang/bottom/pull/919): Add an option to expand the default widget on startup.

### Changes

- [#690](https://github.com/ClementTsang/bottom/pull/690): Add some colour to `-h`/`--help` as part of updating to clap 3.0.
- [#726](https://github.com/ClementTsang/bottom/pull/726): Add ARM musl binary build tasks.
- [#807](https://github.com/ClementTsang/bottom/pull/807): Add more human friendly temperature sensor names for Linux.
- [#845](https://github.com/ClementTsang/bottom/pull/845), [#922](https://github.com/ClementTsang/bottom/pull/922): Add macOS M1, FreeBSD 12, and FreeBSD 13 binary build tasks.
- [#916](https://github.com/ClementTsang/bottom/pull/916), [#937](https://github.com/ClementTsang/bottom/pull/937): Improve CPU usage by optimizing draw logic of charts and tables.

### Bug Fixes

- [#711](https://github.com/ClementTsang/bottom/pull/711): Fix building in Rust beta 1.61 due to `as_ref()` calls causing type inference issues.
- [#717](https://github.com/ClementTsang/bottom/pull/717): Fix clicking on empty space in tables selecting the very last entry of a list in some cases.
- [#720](https://github.com/ClementTsang/bottom/pull/720): Fix panic if battery feature was disabled during compilation.
- [#805](https://github.com/ClementTsang/bottom/pull/805): Fix bottom keeping devices awake in certain scenarios.
- [#825](https://github.com/ClementTsang/bottom/pull/825): Use alternative method of getting parent PID in some cases on macOS devices to avoid needing root access.
- [#916](https://github.com/ClementTsang/bottom/pull/916): Fix possible gaps with widget layout spacing.
- [#938](https://github.com/ClementTsang/bottom/pull/938): Fix search scrolling with wider Unicode characters.

## [0.6.8] - 2022-02-01

### Bug Fixes

- [#655](https://github.com/ClementTsang/bottom/pull/669): Fix a bug where the number of CPUs is never refreshed.

## [0.6.7] - 2022-01-31

### Features

- [#646](https://github.com/ClementTsang/bottom/pull/646): Add `PgUp`/`PgDown` keybind support to scroll up and down a page in a table.

### Bug Fixes

- [#655](https://github.com/ClementTsang/bottom/pull/665): Fix bug where the program would stall in an infinite loop if the width of the terminal was too small.

### Other

- [#658](https://github.com/ClementTsang/bottom/pull/658): Update sysinfo.

## [0.6.6] - 2021-12-22

### Changes

- [#637](https://github.com/ClementTsang/bottom/pull/637): Remove duplicate guest time in process CPU calculation

### Bug Fixes

- [#637](https://github.com/ClementTsang/bottom/pull/637): Fix process CPU calculation if /proc/stat CPU line has fewer values than expected

## [0.6.5] - 2021-12-19

### Bug Fixes

- [#600](https://github.com/ClementTsang/bottom/pull/600): Address RUSTSEC-2020-0071
- [#627](https://github.com/ClementTsang/bottom/pull/627): Fix `process_command` breaking process widget sorting.

### Internal Changes

- [#608](https://github.com/ClementTsang/bottom/pull/608): Add codecov integration to pipeline.

## [0.6.4] - 2021-09-12

### Changes

- [#557](https://github.com/ClementTsang/bottom/pull/557): Add '/s' to network usage legend to better indicate that it's a per-second change.

### Bug Fixes

- [#575](https://github.com/ClementTsang/bottom/pull/575): Updates the procfs library to not crash on kernel version >255.

### Internal Changes

- [#551](https://github.com/ClementTsang/bottom/pull/551): Disable AUR package generation in release pipeline since it's now in community.
- [#570](https://github.com/ClementTsang/bottom/pull/570): Make battery features optional in compilation.

## [0.6.3] - 2021-07-18

### Changes

- [#547](https://github.com/ClementTsang/bottom/pull/547): Switch Linux memory usage calculation to match htop.

### Bug Fixes

- [#536](https://github.com/ClementTsang/bottom/pull/536): Prevent tests from creating a config file.

- [#542](https://github.com/ClementTsang/bottom/pull/542): Fix missing config options in the default generated config file.

- [#545](https://github.com/ClementTsang/bottom/pull/545): Fix inaccurate memory usage/totals in macOS and Linux, switch unit to binary prefix.

## [0.6.2] - 2021-06-26

### Features

- [#518](https://github.com/ClementTsang/bottom/pull/518): Add `F9` key as an alternative process kill key.

### Bug Fixes

- [#504](https://github.com/ClementTsang/bottom/pull/504): Fix two bugs causing the battery widget colours and mouse events to be broken.

- [#525](https://github.com/ClementTsang/bottom/pull/525): Fix Windows process CPU usage not being divided by the number of cores.

### Internal Changes

- [#506](https://github.com/ClementTsang/bottom/pull/506): Migrate a large portion of documentation over to mkdocs.

## [0.6.1] - 2021-05-11

### Bug Fixes

- [#473](https://github.com/ClementTsang/bottom/pull/473): Fix missing string creation for memory usage in collapsed entries.

## [0.6.0] - 2021-05-09

### Features

- [#263](https://github.com/ClementTsang/bottom/pull/263): Add the option for fine-grained kill signals on Unix-like systems.

- [#333](https://github.com/ClementTsang/bottom/pull/333): Add an "out of" indicator that can be enabled using `--show_table_scroll_position` (and its corresponding config option) to help keep track of scrolled position.

- [#379](https://github.com/ClementTsang/bottom/pull/379): Add `--process_command` flag and corresponding config option to default to showing a process' command.

- [#381](https://github.com/ClementTsang/bottom/pull/381): Add a filter in the config file for network interfaces.

- [#392](https://github.com/ClementTsang/bottom/pull/392): Add CPU load averages (1, 5, 15) for Unix-based systems.

- [#406](https://github.com/ClementTsang/bottom/pull/406): Add the Nord colour scheme, as well as a light variant.

- [#409](https://github.com/ClementTsang/bottom/pull/409): Add `Ctrl-w` and `Ctrl-h` shortcuts in search, to delete a word and delete a character respectively.

- [#413](https://github.com/ClementTsang/bottom/pull/413): Add mouse support for sorting process columns.

- [#425](https://github.com/ClementTsang/bottom/pull/425): Add user into the process widget for Unix-based systems.

- [#437](https://github.com/ClementTsang/bottom/pull/437): Redo dynamic network y-axis, add linear scaling, unit type, and prefix options.

- [#445](https://github.com/ClementTsang/bottom/pull/445): Add collapsing in tree mode sums usage to parent.

### Changes

- [#372](https://github.com/ClementTsang/bottom/pull/372): Hide the SWAP graph and legend in normal mode if SWAP is 0.

- [#390](https://github.com/ClementTsang/bottom/pull/390): macOS shouldn't need elevated privileges to see CPU usage on all processes now.

- [#391](https://github.com/ClementTsang/bottom/pull/391): Show degree symbol on Celsius and Fahrenheit.

- [#418](https://github.com/ClementTsang/bottom/pull/418): Removed automatically jumping to the top of the list for process sort shortcuts. The standard behaviour is to now stay in place.

- [#420](https://github.com/ClementTsang/bottom/pull/420): Updated tui-rs, allowing for prettier looking tables!

- [#437](https://github.com/ClementTsang/bottom/pull/437): Add linear interpolation step in drawing step to pr event missing entries on the right side of charts.

- [#443](https://github.com/ClementTsang/bottom/pull/443): Make process widget consistent with disk widget in using decimal prefixes (kilo, mega, etc.) for writes/reads.

- [#449](https://github.com/ClementTsang/bottom/pull/449): Add decimal place to actual memory usage in process widget for values greater or equal to 1GiB.

- [#450](https://github.com/ClementTsang/bottom/pull/450): Tweak `default-light` colour scheme to look less terrible on white terminals.

- [#451](https://github.com/ClementTsang/bottom/pull/451): Add decimal place to disk values larger than 1GB for total read/write in process widgets, and read/write per second in process widgets and disk widgets.

- [#455](https://github.com/ClementTsang/bottom/pull/455): Add a mount point filter for the disk widget. Also tweaked how the filter system works - see the PR for details.

### Bug Fixes

- [#416](https://github.com/ClementTsang/bottom/pull/416): Fix grouped vs ungrouped modes in the processes widget having inconsistent spacing.

- [#417](https://github.com/ClementTsang/bottom/pull/417): Fix the sort menu and sort shortcuts not syncing up.

- [#423](https://github.com/ClementTsang/bottom/pull/423): Fix disk encryption causing the disk widget to fail or not properly map I/O statistics.

- [#425](https://github.com/ClementTsang/bottom/pull/425): Fixed a bug allowing grouped mode in tree mode if already in grouped mode.

- [#467](https://github.com/ClementTsang/bottom/pull/467): Switched CPU usage data source to fix a bug on Windows where occasionally CPU usage would be stuck at 0%.

## [0.5.7] - 2021-01-30

### Bug Fixes

- [#373](https://github.com/ClementTsang/bottom/pull/373): Fix incorrect colours being used the CPU widget in basic mode.

- [#386](https://github.com/ClementTsang/bottom/pull/386): Fix `hide_table_gap` not working in the battery widget.

- [#389](https://github.com/ClementTsang/bottom/pull/389): Fix the sorting arrow disappearing in proc widget under some cases.

- [#398](https://github.com/ClementTsang/bottom/pull/398): Fix basic mode failing to report CPUs if there are less than 4 entries to report.

## [0.5.6] - 2020-12-17

### Bug Fixes

- [#361](https://github.com/ClementTsang/bottom/pull/361): Fix temperature sensors not working on non-Linux platforms.

## [0.5.5] - 2020-12-14

### Bug Fixes

- [#349](https://github.com/ClementTsang/bottom/pull/349): Fix CPU graph colours not matching the legend in the "all" state.

## [0.5.4] - 2020-12-10

### Changes

- [#344](https://github.com/ClementTsang/bottom/pull/344): Removed the `--debug` option for now.

### Bug Fixes

- [#344](https://github.com/ClementTsang/bottom/pull/344): Fix a performance regression causing high memory and CPU usage over time.

- [#345](https://github.com/ClementTsang/bottom/pull/345): Fix process states not showing.

## [0.5.3] - 2020-11-26

### Bug Fixes

- [#331](https://github.com/ClementTsang/bottom/pull/331): Fix custom battery colour levels being inverted.

## [0.5.2] - 2020-11-25

### Bug Fixes

- [#327](https://github.com/ClementTsang/bottom/pull/327): Fix `hide_avg_cpu` being inverted in config files.

## [0.5.1] - 2020-11-22

### Bug Fixes

- [6ef1d66](https://github.com/ClementTsang/bottom/commit/6ef1d66b2bca49452572a2cabb87d338dcf56e7b): Remove nord as a valid colour for now.

- [e04ce4f](https://github.com/ClementTsang/bottom/commit/e04ce4fa1b42e99f00cf8825bcd58da43552214e): Fix `--use_old_network_legend`.

- [99d0402](https://github.com/ClementTsang/bottom/commit/99d04029f0ebfc73d36adb06ea58ad68f090017c): Fix config detection for built-in colours.

## [0.5.0] - 2020-11-20

### Features

- [#206](https://github.com/ClementTsang/bottom/pull/206): Adaptive network graphs --- prior to this update, graphs were stuck at a range from 0B to 1GiB. Now, they adjust to your current usage and time span, so if you're using, say, less than a MiB, it will cap at a MiB. If you're using 10GiB, then the graph will reflect that and span to a bit greater than 10GiB.

- [#208](https://github.com/ClementTsang/bottom/pull/208): Mouse support for tables and moving to widgets.

- [#217](https://github.com/ClementTsang/bottom/pull/217): (Kinda) ARM support.

- [#220](https://github.com/ClementTsang/bottom/pull/220): Add ability to hide specific temperature and disk entries via config.

- [#223](https://github.com/ClementTsang/bottom/pull/223): Add tree mode for processes.

  - [#312](https://github.com/ClementTsang/bottom/pull/312): Add a `tree` flag to default to the tree mode.

- [#269](https://github.com/ClementTsang/bottom/pull/269): Add simple indicator for when data updating is frozen.

- [#296](https://github.com/ClementTsang/bottom/pull/296): Built-in colour themes.

- [#309](https://github.com/ClementTsang/bottom/pull/309): Add a `mem_as_value` flag to default displaying process memory as value rather than percentage.

### Changes

- [#213](https://github.com/ClementTsang/bottom/pull/213), [#214](https://github.com/ClementTsang/bottom/pull/214): Updated help descriptions, added auto-complete generation.

- [#296](https://github.com/ClementTsang/bottom/pull/296): Changed how we do battery theming. We now only set high, medium, and low colours, and we deal with the ratios.

### Bug Fixes

- [#211](https://github.com/ClementTsang/bottom/pull/211): Fix a bug where you could move down in the process widget even if the process widget search was closed.

- [#215](https://github.com/ClementTsang/bottom/pull/215): Add labels to Linux temperature values.

- [#224](https://github.com/ClementTsang/bottom/pull/224): Implements sorting by count. It previously did absolutely nothing.

- [#238](https://github.com/ClementTsang/bottom/pull/238): Fix being able to cause an index out-of-bounds by resizing
  to a smaller terminal _just_ after the program got the terminal size, but right before the terminal started drawing.

- [#238](https://github.com/ClementTsang/bottom/pull/238): Fixed not clearing screen before drawing, which caused issues for some environments.

- [#253](https://github.com/ClementTsang/bottom/pull/253): Fix highlighted entries being stuck in another colour when the widget is not selected.

- [#253](https://github.com/ClementTsang/bottom/pull/253), [#266](https://github.com/ClementTsang/bottom/pull/266): Expanding a widget no longer overrides the widget/dialog title colour.

- [#261](https://github.com/ClementTsang/bottom/pull/261): Fixed process names occasionally showing up as truncated, due to only using `/proc/<PID>/stat` as our data source.

- [#262](https://github.com/ClementTsang/bottom/pull/262): Fixed missing thread termination steps as well as improper polling causing blocking in input thread.

- [#289](https://github.com/ClementTsang/bottom/pull/289): Fixed the CPU basic widget showing incorrect data due to an incorrect offset when displaying the data.

- [#290](https://github.com/ClementTsang/bottom/pull/290): Fixed an incorrect offset affecting the CPU colour when scrolling.

- [#291](https://github.com/ClementTsang/bottom/pull/291): Fixed spacing problems in basic CPU mode.

- [#296](https://github.com/ClementTsang/bottom/pull/296): Fixed an incorrect offset affecting the graph CPU colour mismatching the legend.

- [#296](https://github.com/ClementTsang/bottom/pull/296): Removes an accidental extra comma in one of the headers in the disk widget.

- [#308](https://github.com/ClementTsang/bottom/pull/308): Removes the automatically generated CPU colours method.

## [0.4.7] - 2020-08-26

### Bug Fixes

- [#204](https://github.com/ClementTsang/bottom/pull/204): Fix searching by command name being broken.

## [0.4.6] - 2020-08-25

### Features

- [#179](https://github.com/ClementTsang/bottom/pull/179): Show full command/process path as an option.

- [#183](https://github.com/ClementTsang/bottom/pull/183): Added sorting capabilities to any column.

- [#188](https://github.com/ClementTsang/bottom/pull/188): Add (estimated) memory usage values, toggle this from percent to values for processes with `%`.

- [#196](https://github.com/ClementTsang/bottom/pull/196): Support searching processes by process state.

- Added `WASD` as an alternative widget movement system.

- [#198](https://github.com/ClementTsang/bottom/pull/198): Allow `e` to also escape expanded mode.

### Changes

- [#181](https://github.com/ClementTsang/bottom/pull/181): Changed to just support stable (and newer) Rust, due to library incompatibilities.

- [#182](https://github.com/ClementTsang/bottom/pull/182): For macOS, support `$HOME/Library/Application Support` instead of `$HOME/.config` for config files. For backwards compatibility's sake, for macOS, this will still check `.config` if it exists first, but otherwise, it will default to the new location.

### Bug Fixes

- [#183](https://github.com/ClementTsang/bottom/pull/183): Fixed bug in basic mode where the battery widget was placed incorrectly.

- [#186](https://github.com/ClementTsang/bottom/pull/186): Fixed a bug caused by hitting `Enter` when a process kill fails, breaking future process kills.

- [#187](https://github.com/ClementTsang/bottom/pull/187): Fix bug caused by incorrectly reading the `/proc/{pid}/stats` file.

## [0.4.5] - 2020-07-08

- No changes in this update, just an uptick for Crates.io using the wrong Cargo.lock.

## [0.4.4] - 2020-07-06

### Features

- [#114](https://github.com/ClementTsang/bottom/pull/114): Show process state per process (originally in 0.4.0, moved to later). This only shows if the processes are not merged together; I couldn't think of a nice way to show it when grouped together, unfortunately.

### Changes

- [#156](https://github.com/ClementTsang/bottom/issues/156) - Removal of the `/` CPU core showing in the chart. It felt clunky to use, was not really useful, and hard to work with large core counts.

  Furthermore:

  - `show_disabled_data` option and flag is removed.

  - Average CPU is now on by _default_. You can disable it via `-a, --hide_avg_cpu` or `hide_avg_cpu = true`.

  - Make highlighted CPU persist even if widget is not selected - this should help make it easier to know what CPU you are looking at even if you aren't currently on the CPU widget.

### Bug Fixes

- [#164](https://github.com/ClementTsang/bottom/issues/164) - Fixed a bug where bottom would incorrectly read the wrong values to calculate the read/write columns for processes in Linux.

- [#165](https://github.com/ClementTsang/bottom/issues/165) - Fixed a bug where OR operations in the process query wouldn't properly for some cases.

- The process query should hopefully be a bit more usable now. There were issues with how spaces (which are treated as an AND if it was between keywords, so something like `btm cpu > 0 mem > 0` would look for a process named `btm` with cpu usage > 0 and mem usage > 0). This has been hopefully improved.

## [0.4.3] - 2020-05-15

### Other

- Update sysinfo version that fixes an overflow issue.

## [0.4.2] - 2020-05-11

### Changes

- Automatically hide time axis labels if the widget gets too small.

- Automatically hide table gap if the widget gets too small.

### Bug Fixes

- The `<Space>` character can be used as an "AND" again (properly) in queries. For example:

```bash
(btm cpu > 0) (discord mem > 0)
```

is equivalent to:

```bash
(btm AND cpu > 0) AND (discord AND mem > 0)
```

- [#151](https://github.com/ClementTsang/bottom/issues/151) - Fixed an issue where if the drive I/O label didn't match any disk, the entire disk widget would display nothing.

- Display SWAP and MEM legends even if the total amount is 0 to avoid a weird blank spot in the legend.

## [0.4.1] - 2020-05-05

### Bug Fixes

- [#146](https://github.com/ClementTsang/bottom/pull/146): Fixed a typo in the help menu (credit to [HarHarLinks](https://github.com/HarHarLinks)).

## [0.4.0] - 2020-05-04

### Features

- [#58](https://github.com/ClementTsang/bottom/issues/58): I/O stats per process.

- [#55](https://github.com/ClementTsang/bottom/issues/55): Battery monitoring widget.

- [#134](https://github.com/ClementTsang/bottom/pull/134): `hjkl` movement to delete dialog (credit to [andys8](https://github.com/andys8)).

- [#59](https://github.com/ClementTsang/bottom/issues/59): `Alt-h` and `Alt-l` to move left/right in query (and rest of the app actually).

- [#59](https://github.com/ClementTsang/bottom/issues/59): Added a more advanced querying system.

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

- [#59](https://github.com/ClementTsang/bottom/issues/59): Moved maximization key to `e`, renamed feature to _expanding_ the widget. Done to allow for the `<Enter>` key to be used later for a more intuitive usage.

### Bug Fixes

- Fixed `dd` not working on non-first entries.

- Fixed bug where a single empty row as a layout would crash without a proper warning.
  The behaviour now errors out with a more helpful message.

- Fixed bug where empty widgets in layout would cause widget movement to not work properly when moving vertically.

### Internal changes

- [#38](https://github.com/ClementTsang/bottom/issues/38): Updated arg tests and added config testing.

- Add MSRV, starting with 1.40.0.

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

- [#41](https://github.com/ClementTsang/bottom/issues/41): Fix bug that caused the cursor to go off-screen while searching.

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
