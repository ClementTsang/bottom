# Disk I/O Graph Configuration

The disk I/O graph widget is configured under `[disk_io_graph]`.

## Showing Read/Write Lines

By default, both the read and write rate lines are shown. They can be toggled off independently.

```toml
[disk_io_graph]
# Whether to show the read rate line. Defaults to true.
show_read = true

# Whether to show the write rate line. Defaults to true.
show_write = true
```

## Legend Labels

By default, legend entries are labelled by disk name (e.g. `sda`, `nvme0n1`). To use mount points
instead (e.g. `/`, `/home`):

```toml
[disk_io_graph]
legend = "mount"
```

Valid values are `"disk"` (default) and `"mount"`.

## Logarithmic Scale

The y-axis can be switched to a logarithmic scale:

```toml
[disk_io_graph]
use_log = true
```

## Legend Position

The location of the legend can be set with `legend_position`. Valid values are `none`, `top-left`, `top`, `top-right`,
`left`, `right`, `bottom-left`, `bottom`, and `bottom-right`. Defaults to `top-right`.

```toml
[disk_io_graph]
legend_position = "top-right"
```

## Filtering Entries

You can filter out what entries to show by configuring `[disk_io_graph.name_filter]` and `[disk_io_graph.mount_filter]` to filter by
name and mount point respectively. In particular, you can set a list of things to filter with by setting `list`,
and configure how that list is processed with the other options.

If we wanted to ignoring any entry with a name that matches `/dev/sda`:

```toml
[disk_io_graph.name_filter]
# Whether to ignore any matches. Defaults to true.
is_list_ignored = true

# A list of filters to try and match.
list = ["/dev/sda"]

# Whether to use regex. Defaults to false.
regex = true

# Whether to be case-sensitive. Defaults to false.
case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
whole_word = false
```
