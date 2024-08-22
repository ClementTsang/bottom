# Config File

For persistent configuration, and for certain configuration options, bottom supports config files.

## Default Config File

If no config file argument is given, it will automatically look for a config file at these locations:

| OS      | Default Config Location                                                                                                                    |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| macOS   | `$HOME/Library/Application Support/bottom/bottom.toml`<br/> `$HOME/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml` |
| Linux   | `$HOME/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml`                                                             |
| Windows | `C:\Users\<USER>\AppData\Roaming\bottom\bottom.toml`                                                                                       |

If the config file doesn't exist at the path, bottom will automatically try to create a new config file at the location
with default values.

## JSON Schema

The configuration file also has [JSON Schema](https://json-schema.org/) support to make it easier to manage, if your
IDE/editor supports it.
