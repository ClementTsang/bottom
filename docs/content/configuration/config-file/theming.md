# Theming

!!! Warning

    This section is in progress, and is just copied from the old documentation.

The config file can be used to set custom colours for parts of the application under the `[colors]` object. The following labels are customizable with strings that are hex colours, RGB colours, or specific named colours.

Supported named colours are one of the following strings: `Reset, Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, LightYellow, LightBlue, LightMagenta, LightCyan, White`.

| Labels                          | Details                                                 | Example                                                 |
| ------------------------------- | ------------------------------------------------------- | ------------------------------------------------------- |
| Table header colours            | Colour of table headers                                 | `table_header_color="255, 255, 255"`                    |
| CPU colour per core             | Colour of each core. Read in order.                     | `cpu_core_colors=["#ffffff", "white", "255, 255, 255"]` |
| Average CPU colour              | The average CPU color                                   | `avg_cpu_color="White"`                                 |
| All CPUs colour                 | The colour for the "All" CPU label                      | `all_cpu_color="White"`                                 |
| RAM                             | The colour RAM will use                                 | `ram_color="#ffffff"`                                   |
| SWAP                            | The colour SWAP will use                                | `swap_color="#ffffff"`                                  |
| RX                              | The colour rx will use                                  | `rx_color="#ffffff"`                                    |
| TX                              | The colour tx will use                                  | `tx_color="#ffffff"`                                    |
| Widget title colour             | The colour of the label each widget has                 | `widget_title_color="#ffffff"`                          |
| Border colour                   | The colour of the border of unselected widgets          | `border_color="#ffffff"`                                |
| Selected border colour          | The colour of the border of selected widgets            | `highlighted_border_color="#ffffff"`                    |
| Text colour                     | The colour of most text                                 | `text_color="#ffffff"`                                  |
| Graph colour                    | The colour of the lines and text of the graph           | `graph_color="#ffffff"`                                 |
| Cursor colour                   | The cursor's colour                                     | `cursor_color="#ffffff"`                                |
| Selected text colour            | The colour of text that is selected                     | `scroll_entry_text_color="#ffffff"`                     |
| Selected text background colour | The background colour of text that is selected          | `scroll_entry_bg_color="#ffffff"`                       |
| High battery level colour       | The colour used for a high battery level (100% to 50%)  | `high_battery_color="green"`                            |
| Medium battery level colour     | The colour used for a medium battery level (50% to 10%) | `medium_battery_color="yellow"`                         |
| Low battery level colour        | The colour used for a low battery level (10% to 0%)     | `low_battery_color="red"`                               |
