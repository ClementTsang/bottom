# CPU

## Default CPU Graph Selection

You can configure which CPU graph is shown by default when starting up bottom by setting `cpu.default`.

```toml
[cpu]
# One of "all" (default), "average"/"avg"
default = "average"
```

## Show Decimal Places

You can enable showing a decimal place in CPU usage entries by setting `cpu.show_decimals`:

```toml
[cpu]
show_decimals = true
```
