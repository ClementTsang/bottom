# Processes

## Settings

If you want to change some of the default behaviour of the processes widget, you can configure some things in the config file.

| Field                   | Type    | Functionality                                                                                             |
| ----------------------- | ------- | --------------------------------------------------------------------------------------------------------- |
| `get_threads`           | Boolean | Gather process thread information.                                                                        |
| `hide_k_threads`        | Boolean | Hide kernel threads from being shown.                                                                     |
| `tree_collapse`         | Boolean | Collapse the process tree by default when tree mode is set.                                               |
| `process_command`       | Boolean | Shows the full command name instead of the process name by default.                                       |
| `disable_advanced_kill` | Boolean | Disable the advanced kill dialog and just show the basic one with no options. Linux, macOS, FreeBSD only. |
| `default_memory_value`  | Boolean | Defaults to showing process memory usage by value.                                                        |
| `default_grouped`       | Boolean | Groups processes with the same name by default. No effect if `--tree` is set.                             |
| `regex`                 | Boolean | Enables regex by default while searching.                                                                 |
| `case_sensitive`        | Boolean | Enables case sensitivity by default when searching.                                                       |
| `whole_word`            | Boolean | Enables whole-word matching by default while searching.                                                   |
| `default_tree`          | Boolean | Makes the process widget use tree mode by default.                                                        |
| `current_usage`         | Boolean | Calculates process CPU usage as a percentage of current usage rather than total usage.                    |
| `unnormalized_cpu`      | Boolean | Show process CPU% usage without averaging over the number of CPU cores.                                   |

## Columns

You can configure which columns are shown by the process widget by setting the `columns` setting:

```toml
[processes]
# Pick which columns you want to use in any order.
columns = ["cpu%", "mem%", "pid", "name", "read", "write", "tread", "twrite", "state", "user", "time", "gmem%", "gpu%"]
```

## Default Sort Order

By default, the process widget starts sorted by CPU usage. You can change the column it sorts by at startup:

```toml
[processes]
default_sort = "mem"
```

Any of the column names accepted by `columns` work here (e.g. `"cpu%"`, `"mem"`, `"pid"`, `"name"`, `"read"`). If the
column you pick is not actually shown by the widget, the built-in default is used instead.

The same setting is also exposed as a CLI flag, which takes precedence over the config file:

```
btm --process_default_sort mem
```
