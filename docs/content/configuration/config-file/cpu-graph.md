# CPU Configuration

If you want to change some of the default behaviour of the CPU graph widget, you can configure things under the `[cpu]` section.

## Default CPU graph selection

You can configure which CPU graph is shown by default on startup by setting `cpu.default`. Defaults to `"all"`, which
shows all entries.

```toml
[cpu]
# One of "all" (default), "average"/"avg"
default = "average"
```

## Show decimal

You can configure whether CPU usage values are shown with a decimal place by setting `cpu.show_decimal`. Defaults
to `false`.

```toml
[cpu]
show_decimal = true
```

## Hide average CPU entry

You can hide the average CPU entry entirely by setting `cpu.hide_avg_cpu`. Defaults to `false`.

```toml
[cpu]
hide_avg_cpu = true
```

## Place legend on the left

You can place the CPU chart legend on the left side by setting `cpu.left_legend`. Defaults to `false`.

```toml
[cpu]
left_legend = true
```

## In-chart legend

You can replace the classic side table of per-CPU usages with a compact in-chart legend
(like the memory and network graph widgets) by setting `cpu.legend_position`. When set, the
CPU chart uses the full widget width, and the legend shows the current usage of the
selected entries in a small box drawn over the chart at the configured position.

```toml
[cpu]
# One of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]
legend_position = "top-right"
```

- `"none"` hides the legend entirely (and the side table is not drawn either).
- If left unset, the classic side table behaviour is kept.

When the legend is set, there is no side table, so use the arrow keys while the CPU widget
is selected to change which entry is graphed (the same way you would have scrolled the side
table).

- In the "all" view, the legend labels the average CPU usage (`AVG 12%`), or the first
  per-CPU entry if the average entry is hidden with `--hide_avg_cpu`.
- When a single CPU is selected, it labels that entry (`CPU3 8%`).
- The usage value honours `cpu.show_decimal`, matching the side table.

## Average CPU row

In basic mode, you can give the average CPU entry a dedicated row by setting `cpu.basic_average_cpu_row`. Defaults to `false`.

```toml
[cpu]
basic_average_cpu_row = true
```
