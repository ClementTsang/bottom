# Temperature Graph

The temperature graph widget is configured under `[temperature_graph]`.

## Legend Position

The location of the legend can be set with `legend_position`. Valid values are `none`, `top-left`, `top`, `top-right`,
`left`, `right`, `bottom-left`, `bottom`, and `bottom-right`. Defaults to `top-right`.

```toml
[temperature_graph]
legend_position = "top-right"
```

## Upper Limit

By default, the y-axis is bounded at 100°C (or the equivalent in the configured `temperature_type`) and grows
automatically if a reading exceeds that. An explicit upper bound can be set with `max_temp` (uses the same unit as
`temperature_type`). Sensor readings above this value will be drawn off the chart.

```toml
[temperature_graph]
max_temp = 90.0
```

## Filtering Entries

You can filter which sensors to plot by configuring `[temperature_graph.sensor_filter]`. This works the same way as
the temperature table's filter.

For example, here we are ignoring any sensor that has "cpu" or "wifi" in its name:

```toml
[temperature_graph.sensor_filter]
# Whether to ignore any matches. Defaults to true.
is_list_ignored = true

# A list of filters to try and match.
list = ["cpu", "wifi"]

# Whether to use regex. Defaults to false.
regex = false

# Whether to be case-sensitive. Defaults to false.
case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
whole_word = false
```
