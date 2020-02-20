# Config Files

## Boot Options

One use of a config file is to set boot flags to execute without having to state them when launching the program.

- This is set under the `[flags]` section.
- These options are generally the same as the long names as other flags (ex: `case_sensitive = true`).
- Note that if a flag and an option conflict, the flag has higher precedence (ex: if the `-c` and `temperature_type = kelvin` both exist, the Celsius temperature type is ultimately chosen).
- For temperature type, use `temperature_type = "kelvin|k|celsius|c|fahrenheit|f"`.
- For default widgets, use `default_widget = "cpu_default|memory_default|disk_default|temperature_default|network_default|process_default"`.

## Colours

Another use is to set colours, under the `[colors]`. The following labels are customizable with hex colour code strings:

- Table header colours (`table_header_color="#ffffff"`).
- Every CPU core colour as an array (`cpu_core_colors=["#ffffff", "#000000", "#111111"]`).
  - bottom will look at 216 (let's be realistic here) colours at most, and in order.
  - If not enough colours are provided for the number of threads on the CPU, then the rest will be automatically generated.
- RAM and SWAP colours (`ram_color="#ffffff"`, `swap_color="#111111"`).
- RX and TX colours (`rx_color="#ffffff"`, `tx_color="#111111"`).
- Widget title colour (`widget_title_color="#ffffff"`).
- General widget border colour (`border_color="#ffffff"`).
- Current widget border colour (`highlighted_border_color="#ffffff"`).
- Text colour (`text_color="#ffffff"`).
- Label and graph colour (`graph_color="#ffffff"`).
- Cursor colour (`cursor_color="#ffffff"`).
- Current selected scroll entry colour (`scroll_entry_text_color="#282828"`, `scroll_entry_bg_color="#458588"`).

Note some colours may not be compatible with the terminal you are using.

## Default Locations

bottom will check specific locations by default for a config file.

- For Unix-based systems: `~/.config/btm/btm.toml`.
- For Windows: `./btm.toml`.
