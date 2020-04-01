# Config Files

## Boot options

One use of a config file is to set boot flags to execute without having to state them when launching the program.

- This is set under the `[flags]` section.
- These options are generally the same as the long names as other flags (ex: `case_sensitive = true`).
- Note that if a flag and an option conflict, the flag has higher precedence (ex: if the `-c` and `temperature_type = kelvin` both exist, the Celsius temperature type is ultimately chosen).
- For temperature type, use `temperature_type = "kelvin|k|celsius|c|fahrenheit|f"`.
- For default widgets, use `default_widget = "cpu_default|memory_default|disk_default|temperature_default|network_default|process_default"`.

## Colours

Another use is to set colours, under the `[colors]`. The following labels are customizable with strings that are hex colours, RGB colours, or specific named colours.

Supported named colours are one of the following: `Reset, Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, LightYellow, LightBlue, LightMagenta, LightCyan, White`

| Labels                          | Details                                        | Example                                                |
| ------------------------------- | ---------------------------------------------- | ------------------------------------------------------ |
| Table header colours            | Colour of table headers                        | `table_header_color="256, 256, 256"`                   |
| CPU colour per core             | Colour of each core. Read in order.            | `cpu_core_colors=["#ffffff", "blue", "122, 122, 122"]` |
| Average CPU colour              | The average CPU color                          | `avg_cpu_color="Red"`                                  |
| RAM                             | The colour RAM will use                        | `ram_color="#ffffff"`                                  |
| SWAP                            | The colour SWAP will use                       | `swap_color="#111111"`                                 |
| RX                              | The colour rx will use                         | `rx_color="#ffffff"`                                   |
| TX                              | The colour tx will use                         | `tx_color="#111111"`                                   |
| Widget title colour             | The colour of the label each widget has        | `widget_title_color="#ffffff"`                         |
| Border colour                   | The colour of the border of unselected widgets | `border_color="#ffffff"`                               |
| Selected border colour          | The colour of the border of selected widgets   | `highlighted_border_color="#ffffff"`                   |
| Text colour                     | The colour of most text                        | `text_color="#ffffff"`                                 |
| Graph colour                    | The colour of the lines and text of the graph  | `graph_color="#ffffff"`                                |
| Cursor colour                   | The cursor's colour                            | `cursor_color="#ffffff"`                               |
| Selected text colour            | The colour of text that is selected            | `scroll_entry_text_color="#282828"`                    |
| Selected text background colour | The background colour of text that is selected | `scroll_entry_bg_color="#458588"`                      |

Note some colours may not be compatible with the terminal you are using. For example, macOS's default Terminal does not play nice with many colours.

## Layout

As of 0.3.0, bottom supports custom layouts. Layouts are in the TOML specification, and are arranged by row -> column -> row. For example, the default layout:

```toml
[[row]]
  ratio=30
  [[row.child]]
  type="cpu"
[[row]]
    ratio=40
    [[row.child]]
      ratio=4
      type="mem"
    [[row.child]]
      ratio=3
      [[row.child.child]]
        type="temp"
      [[row.child.child]]
        type="disk"
[[row]]
  ratio=30
  [[row.child]]
    type="net"
  [[row.child]]
    type="proc"
    default=true
```

Valid types are:

- `cpu`
- `mem`
- `proc`
- `net`
- `temp`
- `disk`
- `empty`

## Default config locations

bottom will check specific locations by default for a config file. If no file is found, it will be created.

- For Unix-based systems: `$HOME/.config/bottom/bottom.toml`.
- For Windows: `{FOLDERID_RoamingAppData}\bottom\bottom.toml` (for example, `C:\Users\Clement\AppData\Roaming\bottom\bottom.toml`).
