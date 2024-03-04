# Command-line Options

The following options can be provided to bottom in the command line to change the behaviour of the program. You can also
see information on these options by running `btm -h`, or run `btm --help` to display more detailed information on each option:

## General Options

| Option                                | Behaviour                                           |
| ------------------------------------- | --------------------------------------------------- |
| `--autohide_time`                     | Temporarily shows the time scale in graphs.         |
| `-b`, `--basic`                       | Hides graphs and uses a more basic look.            |
| `-C`, `--config <CONFIG PATH>`        | Sets the location of the config file.               |
| `-t`, `--default_time_value <TIME>`   | Default time value for graphs.                      |
| `--default_widget_count <INT>`        | Sets the n'th selected widget type as the default.  |
| `--default_widget_type <WIDGET TYPE>` | Sets the default widget type, use --help for info.  |
| `--disable_click`                     | Disables mouse clicks.                              |
| `-m`, `--dot_marker`                  | Uses a dot marker for graphs.                       |
| `-e`, `--expanded`                    | Expand the default widget upon starting the app.    |
| `--hide_table_gap`                    | Hides spacing between table headers and entries.    |
| `--hide_time`                         | Hides the time scale.                               |
| `-l`, `--left_legend`                 | Puts the CPU chart legend to the left side.         |
| `-r`, `--rate <TIME>`                 | Sets the data refresh rate.                         |
| `--retention <TIME>`                  | The timespan of data stored.                        |
| `--show_table_scroll_position`        | Shows the scroll position tracker in table widgets. |
| `-d`, `--time_delta <TIME>`           | The amount of time changed upon zooming.            |

## Process Options

| Option                     | Behaviour                                                             |
| -------------------------- | --------------------------------------------------------------------- |
| `-S`, `--case_sensitive`   | Enables case sensitivity by default.                                  |
| `-u`, `--current_usage`    | Sets process CPU% to be based on current CPU%.                        |
| `--disable_advanced_kill`  | Hides advanced process killing.                                       |
| `-g`, `--group_processes`  | Groups processes with the same name by default.                       |
| `--process_command`        | Show processes as their commands by default.                          |
| `-R`, `--regex`            | Enables regex by default.                                             |
| `-T`, `--tree`             | Defaults the process widget be in tree mode.                          |
| `-n`, `--unnormalized_cpu` | Show process CPU% usage without normalizing over the number of cores. |
| `-W`, `--whole_word`       | Enables whole-word matching by default.                               |

## Temperature Options

| Option               | Behaviour                               |
| -------------------- | --------------------------------------- |
| `-c`, `--celsius`    | Use Celsius as the temperature unit.    |
| `-f`, `--fahrenheit` | Use Fahrenheit as the temperature unit. |
| `-k`, `--kelvin`     | Use Kelvin as the temperature unit.     |

## CPU Options

| Option                 | Behaviour                    |
| ---------------------- | ---------------------------- |
| `-a`, `--hide_avg_cpu` | Hides the average CPU usage. |

## Memory Options

| Option                  | Behaviour                                                 |
| ----------------------- | --------------------------------------------------------- |
| `--enable_cache_memory` | Enable collecting and displaying cache and buffer memory. |
| `--mem_as_value`        | Defaults to showing process memory usage by value.        |

## Network Options

| Option                        | Behaviour                                         |
| ----------------------------- | ------------------------------------------------- |
| `--network_use_binary_prefix` | Displays the network widget with binary prefixes. |
| `--network_use_bytes`         | Displays the network widget using bytes.          |
| `--network_use_log`           | Displays the network widget with a log scale.     |
| `--use_old_network_legend`    | DEPRECATED - uses a separate network legend.      |

## Battery Options

| Option      | Behaviour                 |
| ----------- | ------------------------- |
| `--battery` | Shows the battery widget. |

## GPU Options

| Option         | Behaviour                                   |
| -------------- | ------------------------------------------- |
| `--enable_gpu` | Enable collecting and displaying GPU usage. |

## Style Options

| Option                   | Behaviour                                |
| ------------------------ | ---------------------------------------- |
| `--color <SCHEME>` | Use a color scheme, use --help for info. |

## Other Options

| Option            | Behaviour                                  |
| ----------------- | ------------------------------------------ |
| `-h`, `--help`    | Prints help (see more info with '--help'). |
| `-V`, `--version` | Prints version information.                |
