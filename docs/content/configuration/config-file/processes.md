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
