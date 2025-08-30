# Temperature Table

## Filtering Entries

You can filter out what entries to show by configuring `[temperature.sensor_filter]`. In particular you can set a list of things to filter with by setting `list`, and configure how that list is processed with the other options.

For example, here we are ignoring any sensor that has "cpu" or "wifi" in it.

```toml
[temperature.sensor_filter]
# Whether to ignore any matches. Defaults to true.
is_list_ignored = true

# A list of filters to try and match.
list = ["cpu", "wifi"]

# Whether to use regex. Defaults to false.
regex = false

# Whether to be case-sensitive. Defaults to false.
case_sensitive = false

# Whether to be require matching the whole word. Defaults to false.
whole_word = false
```
