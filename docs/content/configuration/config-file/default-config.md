# Default Config

A default config file is automatically generated at the following locations that bottom checks by default:

| OS      | Default Config Location                                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| macOS   | `$HOME/Library/Application Support/bottom/bottom.toml`<br/> `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml` |
| Linux   | `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml`                                                             |
| Windows | `C:\Users\<USER>\AppData\Roaming\bottom\bottom.toml`                                                                                   |

Furthermore, if a custom config path that does not exist is given (using `-C` or `--config`), bottom will attempt to create a default config file at that location.

The configuration file also has [JSON Schema](https://json-schema.org/) support for file validation and completions if your editor supports it.

![completion](https://github.com/ClementTsang/bottom/assets/32936898/cefd2037-d741-4def-98ee-4f77c1713e30)

![diagnostic](https://github.com/ClementTsang/bottom/assets/32936898/4bb5750b-1a07-418e-8b69-8ec30ecf8f82)
