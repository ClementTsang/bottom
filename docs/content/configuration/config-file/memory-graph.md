# Memory Graph

If you want to change some of the default behaviour of the memory graph widget, you can configure things under the `[memory_graph]` (or `[memory]`) section.

## Graph legend position

You can change where the legend for the graph is placed within the widget itself (or hidden with `"none"`). The default is `"top-right"`.

```toml
[memory_graph]
# One of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]
legend_position = "top-left"
```

## Collect/show cache memory

On Linux, you can change whether the memory used by [cache/slabs](https://serverfault.com/a/1025189) is collected and
shown. By default, it is `false`.

```toml
[memory_graph]
cache_memory = true
```

## Subtract free-able ARC from memory

If ZFS is detected (note that the `zfs` feature must be enabled if built manually), you can enable `memory_graph.free_arc` to
not count [ARC](https://www.45drives.com/community/articles/zfs-caching/) memory in the RAM usage calculations. Disabled
by default.

```toml
[memory_graph]
free_arc = true
```
