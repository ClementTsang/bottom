# Default Config

A default config file is automatically generated at the following locations that bottom checks by default:

| OS      | Default Config Location                                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| macOS   | `$HOME/Library/Application Support/bottom/bottom.toml`<br/> `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml` |
| Linux   | `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml`                                                             |
| Windows | `C:\Users\<USER>\AppData\Roaming\bottom\bottom.toml`                                                                                   |

Furthermore, if a custom config path that does not exist is given (using `-C` or `--config`), bottom will attempt to create a default config file at that location.
