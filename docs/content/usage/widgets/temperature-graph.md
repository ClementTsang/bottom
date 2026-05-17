# Temperature Graph Widget

The temperature graph widget provides temperature readings over time.

## Features

Each detected sensor is drawn as its own line. The y-axis is in the configured temperature unit (Celsius by default;
see the `--celsius`/`--fahrenheit`/`--kelvin` arguments or `flags.temperature_type` config option).

By default the y-axis is bounded at 100°C (or the equivalent in the configured unit) and grows automatically. An upper bound can also be set explicitly via the [config file](../../configuration/config-file/temperature-graph.md).

The displayed time range can be adjusted through either the keyboard or mouse.

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding   | Action                                  |
| --------- | --------------------------------------- |
| ++plus++  | Zoom in on chart (decrease time range)  |
| ++minus++ | Zoom out on chart (increase time range) |
| ++equal++ | Reset zoom                              |

## Mouse bindings

| Binding      | Action                                                         |
| ------------ | -------------------------------------------------------------- |
| ++"Scroll"++ | Scrolling up or down zooms in or out of the graph respectively |

<!--TODO: Add screenshots of widgets -->
