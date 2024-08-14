# Command-line Options

The following options can be provided to bottom in the command line to change the behaviour of the program. You can also
see information on these options by running `btm -h`, or run `btm --help` to display more detailed information on each option:

## General Options

| Option                            | Behaviour                                            |
| --------------------------------- | ---------------------------------------------------- |
| `--autohide_time`                 | Temporarily shows the time scale in graphs.          |
| `-b, --basic`                     | Hides graphs and uses a more basic look.             |
| `-C, --config <CONFIG PATH>`      | Sets the location of the config file.                |
| `-t, --default_time_value <TIME>` | Default time value for graphs.                       |
| `--default_widget_count <N>`      | Sets the N'th selected widget type as the default.   |
| `--default_widget_type <WIDGET>`  | Sets the default widget type, use `--help` for info. |
| `--disable_click`                 | Disables mouse clicks.                               |
| `-m, --dot_marker`                | Uses a dot marker for graphs.                        |
| `-e, --expanded`                  | Expand the default widget upon starting the app.     |
| `--hide_table_gap`                | Hides spacing between table headers and entries.     |
| `--hide_time`                     | Hides the time scale from being shown.               |
| `-r, --rate <TIME>`               | Sets how often data is refreshed.                    |
| `--retention <TIME>`              | How far back data will be stored up to.              |
| `--show_table_scroll_position`    | Shows the scroll position tracker in table widgets.  |
| `-d, --time_delta <TIME>`         | The amount of time changed upon zooming.             |

## Process Options

| Option                      | Behaviour                                                                              |
| --------------------------- | -------------------------------------------------------------------------------------- |
| `-S, --case_sensitive`      | Enables case sensitivity by default.                                                   |
| `-u, --current_usage`       | Calculates process CPU usage as a percentage of current usage rather than total usage. |
| `--disable_advanced_kill`   | Hides additional stopping options Unix-like systems.                                   |
| `-g, --group_processes`     | Groups processes with the same name by default.                                        |
| `--process_memory_as_value` | Defaults to showing process memory usage by value.                                     |
| `--process_command`         | Shows the full command name instead of the process name by default.                    |
| `-R, --regex`               | Enables regex by default while searching.                                              |
| `-T, --tree`                | Makes the process widget use tree mode by default.                                     |
| `-n, --unnormalized_cpu`    | Show process CPU% usage without averaging over the number of CPU cores.                |
| `-W, --whole_word`          | Enables whole-word matching by default while searching.                                |

## Temperature Options

| Option             | Behaviour                                     |
| ------------------ | --------------------------------------------- |
| `-c, --celsius`    | Use Celsius as the temperature unit. Default. |
| `-f, --fahrenheit` | Use Fahrenheit as the temperature unit.       |
| `-k, --kelvin`     | Use Kelvin as the temperature unit.           |

## CPU Options

| Option                | Behaviour                                         |
| --------------------- | ------------------------------------------------- |
| `--cpu_left_legend`   | Puts the CPU chart legend on the left side.       |
| `--default_cpu_entry` | Sets which CPU entry type is selected by default. |
| `-a, --hide_avg_cpu`  | Hides the average CPU usage entry.                |

## Memory Options

| Option                       | Behaviour                                                 |
| ---------------------------- | --------------------------------------------------------- |
| `--memory_legend <POSITION>` | Where to place the legend for the memory chart widget.    |
| `--enable_cache_memory`      | Enable collecting and displaying cache and buffer memory. |

## Network Options

| Option                        | Behaviour                                               |
| ----------------------------- | ------------------------------------------------------- |
| `--network_legend <POSITION>` | Where to place the legend for the network chart widget. |
| `--network_use_bytes`         | Displays the network widget using bytes.                |
| `--network_use_binary_prefix` | Displays the network widget with binary prefixes.       |
| `--network_use_log`           | Displays the network widget with a log scale.           |
| `--use_old_network_legend`    | (DEPRECATED) Uses a separate network legend.            |

## Battery Options

| Option      | Behaviour                                       |
| ----------- | ----------------------------------------------- |
| `--battery` | Shows the battery widget in non-custom layouts. |

## GPU Options

| Option          | Behaviour                                                 |
| --------------- | --------------------------------------------------------- |
| `--disable_gpu` | Disable collecting and displaying NVIDIA GPU information. |

## Style Options

| Option             | Behaviour                                                        |
| ------------------ | ---------------------------------------------------------------- |
| `--theme <SCHEME>` | Use a built-in color theme, use '--help' for info on the colors. |

## Other Options

| Option            | Behaviour                                         |
| ----------------- | ------------------------------------------------- |
| `-h`, `--help`    | Prints help info (for more details use `--help`.) |
| `-V`, `--version` | Prints version information.                       |
