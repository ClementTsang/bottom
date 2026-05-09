# Processes

## Settings

If you want to change some of the default behaviour of the processes widget, you can configure some things in the config file.

| Field         | Type    | Functionality                      |
| ------------- | ------- | ---------------------------------- |
| `get_threads` | Boolean | Gather process thread information. |

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
