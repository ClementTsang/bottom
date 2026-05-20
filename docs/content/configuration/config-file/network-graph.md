# Network Graph

## Settings

If you want to change some of the default behaviour of the network graph widget, you can configure some things in the config file.

| Field               | Type                                                                                                               | Functionality                                                                                                                                |
| ------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `show_packets`      | Boolean                                                                                                            | Displays packet rate and average packet size info.                                                                                           |
| `legend_position`   | String (one of ["none", "top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"]) | Where to place the legend for the network widget.                                                                                            |
| `use_bytes`         | Boolean                                                                                                            | Displays the network widget using bytes. Defaults to bits.                                                                                   |
| `use_log`           | Boolean                                                                                                            | Displays the network widget with a log scale. Defaults to a non-log scale.                                                                   |
| `use_binary_prefix` | Boolean                                                                                                            | Displays the network widget with a binary prefix (e.g. kibibits) rather than a decimal prefix (e.g. kilobits). Defaults to decimal prefixes. |

## Filtering Entries

You can filter out what entries to show by configuring `[network_graph.interface_filter]` .
In particular, you can set a list of things to filter with by setting `list`, and configure how that list is processed with the other options.

For example, here we are ignoring any entry with a name that matches the regex `eth0.*`, or specifically `virbr0`.

```toml
[network_graph.interface_filter]
# Whether to ignore any matches. Defaults to true.
is_list_ignored = true

# A list of filters to try and match.
list = ["virbr0", "eth0slab"]

# Whether to use regex. Defaults to false.
regex = true

# Whether to be case-sensitive. Defaults to false.
case_sensitive = false

# Whether to require matching the whole word. Defaults to false.
whole_word = false
```
