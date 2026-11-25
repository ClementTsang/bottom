# Memory Graph Configuration

If you want to change some of the default behaviour of the memory graph widget, you can configure things under the `[memory_graph]` (or `[memory]`) section.

## Graph legend position

You can change where the legend for the graph is placed within the widget itself (or hidden with `"none"`). The default is `"top-right"`.

```toml
[memory_graph]
# One of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]
legend_position = "top-left"
```

## Collect and show cache memory

On Linux, you can change whether the memory used by [cache/slabs](https://serverfault.com/a/1025189) is collected and
shown. By default, it is `false`.

<!--TODO: Note this affects basic too... should I separate out a "general memory" setting and specific graph ones?-->

```toml
[memory_graph]
cache_memory = true
```

## Collect and show zram usage

On Linux, you can enable `zram` to show [zram](https://docs.kernel.org/admin-guide/blockdev/zram.html) usage.
Usage is over all zram devices and is shown as the uncompressed size of the stored data versus the total uncompressed
capacity. Also note that if zram is used as swap, this usage is currently also included in the swap
usage.

It is disabled (`false`) by default. For example, to enable it:

```toml
[memory_graph]
zram = true
```

## Disable GPU usage

GPU memory usage is collected by default. To disable it, do:

```toml
[flags]
disable_gpu = true
```

Note that for now, this disables all other GPU features, though this is likely to change in the future to be more granular.

## Short GPU names

If GPU support is enabled, you can use `short_gpu_names` to display simplified GPU names in the memory graph widget
instead of the full GPU names. If enabled, then if there's only one GPU, it'll just list the entry as `GPU`;
if there are multiple GPUs, it'll use entries like `GPU0`, `GPU1`, etc. By default, this is disabled.

```toml
[memory_graph]
short_gpu_names = true
```

## Subtract free-able ARC from memory

If ZFS is detected (note that the `zfs` feature must be enabled if built manually), you can enable `memory_graph.free_arc` to
not count [ARC](https://www.45drives.com/community/articles/zfs-caching/) memory in the RAM usage calculations. Disabled
by default.

```toml
[memory_graph]
free_arc = true
```
