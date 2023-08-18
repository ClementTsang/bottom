# Processes

## Columns

You can configure which columns are shown by the process widget by setting the `columns` setting:

```toml
[processes]
# Pick which columns you want to use in any order.
columns = ["cpu%", "mem%", "pid", "name", "read", "write", "Tread", "twrite", "state", "user", "time", "gmem", "gpu%"]
```
