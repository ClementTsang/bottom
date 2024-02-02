# Config File

For persistent configuration, and for certain configuration options, bottom supports config files.

## Default Config File

If no config file argument is given, it will automatically look for a config file at these locations:

| OS      | Default Config Location                                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| macOS   | `$HOME/Library/Application Support/bottom/bottom.toml`<br/> `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml` |
| Linux   | `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml`                                                             |
| Windows | `C:\Users\<USER>\AppData\Roaming\bottom\bottom.toml`                                                                                   |

Like if a path is passed with `-C`/`--config`, if a file doesn't exist at the path, bottom will automatically create a
new, default config file at that location.

## JSON Schema

The configuration file also has [JSON Schema](https://json-schema.org/) support to make it easier to manage, if your
IDE/editor supports it.
