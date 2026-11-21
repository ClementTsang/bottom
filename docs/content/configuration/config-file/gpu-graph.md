# GPU Usage Graph Widget Configuration

If you want to change some of the default behaviour of the GPU usage graph widget, you can configure things under the `[gpu_graph]` section.

## Graph legend position

You can change where the legend for the graph is placed within the widget itself (or hidden with `"none"`). The default is `"top-right"`.

```toml
[gpu_graph]
# One of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]
legend_position = "top-left"
```

## Short GPU names

If GPU support is enabled, you can use `short_gpu_names` to display simplified GPU names in the GPU usage widget
instead of the full GPU names. If enabled, then if there's only one GPU, it'll just list the entry as `GPU`;
if there are multiple GPUs, it'll use entries like `GPU0`, `GPU1`, etc. By default, this is disabled.

```toml
[gpu_graph]
short_gpu_names = true
```
