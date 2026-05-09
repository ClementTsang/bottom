# CPU

## Default CPU Graph Selection

You can configure which CPU graph is shown by default when starting up bottom by setting `cpu.default`.

```toml
[cpu]
# One of "all" (default), "average"/"avg"
default = "average"
```

## Show Decimal

You can configure whether CPU usage values are shown with a decimal place by setting `cpu.show_decimal`. Defaults to `false`.

```toml
[cpu]
show_decimal = true
```
