# Command-line Flags

The following flags can be provided to bottom in the command line to change the behaviour of the program. You can also
see information on these flags by running `btm -h`, or run `btm --help` to display more detailed information on each flag:

| Flag                                | Behaviour                                                             |
| ----------------------------------- | --------------------------------------------------------------------- |
| --autohide_time                     | Temporarily shows the time scale in graphs.                           |
| -b, --basic                         | Hides graphs and uses a more basic look.                              |
| --battery                           | Shows the battery widget.                                             |
| -S, --case_sensitive                | Enables case sensitivity by default.                                  |
| -c, --celsius                       | Sets the temperature type to Celsius.                                 |
| --color <COLOR SCHEME>              | Use a color scheme, use --help for info.                              |
| -C, --config <CONFIG PATH>          | Sets the location of the config file.                                 |
| -u, --current_usage                 | Sets process CPU% to be based on current CPU%.                        |
| -t, --default_time_value <TIME>     | Default time value for graphs.                                        |
| --default_widget_count <INT>        | Sets the n'th selected widget type as the default.                    |
| --default_widget_type <WIDGET TYPE> | Sets the default widget type, use --help for info.                    |
| --disable_advanced_kill             | Hides advanced process killing.                                       |
| --disable_click                     | Disables mouse clicks.                                                |
| -m, --dot_marker                    | Uses a dot marker for graphs.                                         |
| --enable_cache_memory               | Enable collecting and displaying cache and buffer memory.             |
| --enable_gpu                        | Enable collecting and displaying GPU usage.                           |
| -e, --expanded                      | Expand the default widget upon starting the app.                      |
| -f, --fahrenheit                    | Sets the temperature type to Fahrenheit.                              |
| -g, --group_processes               | Groups processes with the same name by default.                       |
| -a, --hide_avg_cpu                  | Hides the average CPU usage.                                          |
| --hide_table_gap                    | Hides spacing between table headers and entries.                      |
| --hide_time                         | Hides the time scale.                                                 |
| -k, --kelvin                        | Sets the temperature type to Kelvin.                                  |
| -l, --left_legend                   | Puts the CPU chart legend to the left side.                           |
| --mem_as_value                      | Defaults to showing process memory usage by value.                    |
| --network_use_binary_prefix         | Displays the network widget with binary prefixes.                     |
| --network_use_bytes                 | Displays the network widget using bytes.                              |
| --network_use_log                   | Displays the network widget with a log scale.                         |
| --process_command                   | Show processes as their commands by default.                          |
| -r, --rate <TIME>                   | Sets the data refresh rate.                                           |
| -R, --regex                         | Enables regex by default.                                             |
| --retention <TIME>                  | The timespan of data stored.                                          |
| --show_table_scroll_position        | Shows the scroll position tracker in table widgets.                   |
| -d, --time_delta <TIME>             | The amount of time changed upon zooming.                              |
| -T, --tree                          | Defaults the process widget be in tree mode.                          |
| -n, --unnormalized_cpu              | Show process CPU% usage without normalizing over the number of cores. |
| --use_old_network_legend            | DEPRECATED - uses a separate network legend.                          |
| -V, --version                       | Prints version information.                                           |
| -W, --whole_word                    | Enables whole-word matching by default.                               |
| -h, --help                          | Print help (see more with '--help')                                   |
