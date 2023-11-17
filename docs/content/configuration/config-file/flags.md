# Flags

!!! Warning

    This section is in progress, and is just copied from the old documentation.

Most of the [command line flags](../command-line-flags.md) have config file equivalents to avoid having to type them out
each time:

| Field                        | Type                                                                                           | Functionality                                                                        |
| ---------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `hide_avg_cpu`               | Boolean                                                                                        | Hides the average CPU usage.                                                         |
| `dot_marker`                 | Boolean                                                                                        | Uses a dot marker for graphs.                                                        |
| `left_legend`                | Boolean                                                                                        | Puts the CPU chart legend to the left side.                                          |
| `current_usage`              | Boolean                                                                                        | Sets process CPU% to be based on current CPU%.                                       |
| `group_processes`            | Boolean                                                                                        | Groups processes with the same name by default.                                      |
| `case_sensitive`             | Boolean                                                                                        | Enables case sensitivity by default.                                                 |
| `whole_word`                 | Boolean                                                                                        | Enables whole-word matching by default.                                              |
| `regex`                      | Boolean                                                                                        | Enables regex by default.                                                            |
| `basic`                      | Boolean                                                                                        | Hides graphs and uses a more basic look.                                             |
| `use_old_network_legend`     | Boolean                                                                                        | DEPRECATED - uses the older network legend.                                          |
| `battery`                    | Boolean                                                                                        | Shows the battery widget.                                                            |
| `rate`                       | Unsigned Int (represents milliseconds) or String (represents human time)                       | Sets a refresh rate in ms.                                                           |
| `default_time_value`         | Unsigned Int (represents milliseconds) or String (represents human time)                       | Default time value for graphs in ms.                                                 |
| `time_delta`                 | Unsigned Int (represents milliseconds) or String (represents human time)                       | The amount in ms changed upon zooming.                                               |
| `hide_time`                  | Boolean                                                                                        | Hides the time scale.                                                                |
| `temperature_type`           | String (one of ["k", "f", "c", "kelvin", "fahrenheit", "celsius"])                             | Sets the temperature unit type.                                                      |
| `default_widget_type`        | String (one of ["cpu", "proc", "net", "temp", "mem", "disk"], same as layout options)          | Sets the default widget type, use --help for more info.                              |
| `default_widget_count`       | Unsigned Int (represents which `default_widget_type`)                                          | Sets the n'th selected widget type as the default.                                   |
| `disable_click`              | Boolean                                                                                        | Disables mouse clicks.                                                               |
| `color`                      | String (one of ["default", "default-light", "gruvbox", "gruvbox-light", "nord", "nord-light"]) | Use a color scheme, use --help for supported values.                                 |
| `enable_cache_memory`        | Boolean                                                                                        | Enable collecting and displaying cache and buffer memory (not available on Windows). |
| `mem_as_value`               | Boolean                                                                                        | Defaults to showing process memory usage by value.                                   |
| `tree`                       | Boolean                                                                                        | Defaults to showing the process widget in tree mode.                                 |
| `show_table_scroll_position` | Boolean                                                                                        | Shows the scroll position tracker in table widgets.                                  |
| `process_command`            | Boolean                                                                                        | Show processes as their commands by default.                                         |
| `disable_advanced_kill`      | Boolean                                                                                        | Hides advanced options to stop a process on Unix-like systems.                       |
| `network_use_binary_prefix`  | Boolean                                                                                        | Displays the network widget with binary prefixes.                                    |
| `network_use_bytes`          | Boolean                                                                                        | Displays the network widget using bytes.                                             |
| `network_use_log`            | Boolean                                                                                        | Displays the network widget with a log scale.                                        |
| `enable_gpu`                 | Boolean                                                                                        | Shows the GPU widgets.                                                               |
| `retention`                  | String (human readable time, such as "10m", "1h", etc.)                                        | How much data is stored at once in terms of time.                                    |
| `unnormalized_cpu`           | Boolean                                                                                        | Show process CPU% without normalizing over the number of cores.                      |
| `expanded_on_startup`        | Boolean                                                                                        | Expand the default widget upon starting the app.                                     |
