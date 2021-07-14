# Command-line Flags

!!! Warning

    This section is in progress, and is just copied from the old documentation.

The following flags can be provided to bottom in the command line to change the behaviour of the program (run `btm --help` for more information on each flag):

| Flag                                  | Behaviour                                                      |
| ------------------------------------- | -------------------------------------------------------------- |
| `--autohide_time`                     | Temporarily shows the time scale in graphs.                    |
| `-b, --basic`                         | Hides graphs and uses a more basic look.                       |
| `--battery`                           | Shows the battery widget.                                      |
| `-S, --case_sensitive`                | Enables case sensitivity by default.                           |
| `-c, --celsius`                       | Sets the temperature type to Celsius.                          |
| `--color <COLOR SCHEME>`              | Use a color scheme, use --help for supported values.           |
| `-C, --config <CONFIG PATH>`          | Sets the location of the config file.                          |
| `-u, --current_usage`                 | Sets process CPU% to be based on current CPU%.                 |
| `-t, --default_time_value <MS>`       | Default time value for graphs in ms.                           |
| `--default_widget_count <INT>`        | Sets the n'th selected widget type as the default.             |
| `--default_widget_type <WIDGET TYPE>` | Sets the default widget type, use --help for more info.        |
| `--disable_advanced_kill`             | Hides advanced options to stop a process on Unix-like systems. |
| `--disable_click`                     | Disables mouse clicks.                                         |
| `-m, --dot_marker`                    | Uses a dot marker for graphs.                                  |
| `-f, --fahrenheit`                    | Sets the temperature type to Fahrenheit.                       |
| `-g, --group`                         | Groups processes with the same name by default.                |
| `-h, --help`                          | Prints help information. Use --help for more info.             |
| `-a, --hide_avg_cpu`                  | Hides the average CPU usage.                                   |
| `--hide_table_gap`                    | Hides the spacing between table headers and entries.           |
| `--hide_time`                         | Hides the time scale.                                          |
| `-k, --kelvin`                        | Sets the temperature type to Kelvin.                           |
| `-l, --left_legend`                   | Puts the CPU chart legend to the left side.                    |
| `--mem_as_value`                      | Defaults to showing process memory usage by value.             |
| `--network_use_binary_prefix`         | Displays the network widget with binary prefixes.              |
| `--network_use_bytes`                 | Displays the network widget using bytes.                       |
| `--network_use_log`                   | Displays the network widget with a log scale.                  |
| `--process_command`                   | Show processes as their commands by default.                   |
| `-r, --rate <MS>`                     | Sets a refresh rate in ms.                                     |
| `-R, --regex`                         | Enables regex by default.                                      |
| `--show_table_scroll_position`        | Shows the scroll position tracker in table widgets.            |
| `-d, --time_delta <MS>`               | The amount in ms changed upon zooming.                         |
| `-T, --tree`                          | Defaults to showing the process widget in tree mode.           |
| `--use_old_network_legend`            | DEPRECATED - uses the older network legend.                    |
| `-V, --version`                       | Prints version information.                                    |
| `-W, --whole_word`                    | Enables whole-word matching by default.                        |
