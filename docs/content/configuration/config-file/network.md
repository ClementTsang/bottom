# Network

## Filtering Entries

You can filter out what entries to show by configuring `[network.interface_filter]` .
In particular, you can set a list of things to filter with by setting `list`, and configure how that list is processed with the other options.

For example, here we are ignoring any entry with a name that matches `/dev/sda<NUMBERS>`, or specifically `/dev/nvme0n1p2`.

```toml
[network.interface_filter]
# Whether to ignore any matches. Defaults to true.
is_list_ignored = true

# A list of filters to try and match.
list = ["virbr0.*"]

# Whether to use regex. Defaults to false.
regex = true

# Whether to be case-sensitive. Defaults to false.
case_sensitive = false

# Whether to be require matching the whole word. Defaults to false.
whole_word = false
```
