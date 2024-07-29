# Theming

Various parts of the application can be styled.

## Built-in styles

bottom has a few built-in themes:

- Default
- [Nord](https://www.nordtheme.com/)
- [Gruvbox](https://github.com/morhetz/gruvbox)

These themes all also have light variants to support terminals using lighter colours.

To set the theme from the command line:

```bash
btm --theme gruvbox
```

To set the theme using the config file:

```toml
[styles]
theme = "gruvbox"
```

## Custom styling

bottom's components can also be individually styled by the user to control the colour of the text style.

### Colours

You can configure the colours for components with strings that are either hex colours (e.g. `"#ffffff"`), RGB colours
(e.g. `"255, 255, 255"`), or named colours. Named colours are one of the following strings:

- `"Black"`
- `"Red"`
- `"Green"`
- `"Yellow"`
- `"Blue"`
- `"Magenta"`
- `"Cyan"`
- `"Gray"`
- `"DarkGray"`
- `"LightRed"`
- `"LightGreen"`
- `"LightYellow"`
- `"LightBlue"`
- `"LightMagenta"`
- `"LightCyan"`
- `"White"`

### Text

Text can generally be styled using the following TOML table:

```toml
[field]
# The foreground colour of text.
color = "black"

# The background colour of text.
bg_color = "blue"

# Whether to make the text bold.
bold = false

# Inline table version
field = { color = "black", bg_color = "blue", bold = false }
```

All fields are optional; by default if `bg_color` is not set then there will be no background color.

### Configuration

#### CPU

These can be set under `[styles.cpu]`:

| Config field      | Details                                                          | Examples                                     |
| ----------------- | ---------------------------------------------------------------- | -------------------------------------------- |
| `all_entry_color` | The colour of the "All" CPU label                                | `all_entry_color = "Red"`                    |
| `avg_entry_color` | The colour of the average CPU label and graph line               | `avg_entry_color = "255, 0, 255"`            |
| `cpu_core_colors` | Colour of each CPU threads' label and graph line. Read in order. | `cpu_core_colors = ["Red", "Blue", "Green"]` |

#### Memory

These can be set under `[styles.memory]`:

| Config field | Details                                                                        | Examples                          |
| ------------ | ------------------------------------------------------------------------------ | --------------------------------- |
| `ram`        | The colour of the RAM label and graph line                                     | `ram = "Red"`                     |
| `cache`      | The colour of the cache label and graph line. Does not do anything on Windows. | `cache = "#ffffff"`               |
| `swap`       | The colour of the swap label and graph line                                    | `swap = "255, 0, 255"`            |
| `arc`        | The colour of the ARC label and graph line                                     | `arc = "Blue"`                    |
| `gpus`       | Colour of each GPU's memory label and graph line. Read in order.               | `gpus = ["Red", "Blue", "Green"]` |

#### Network

These can be set under `[styles.network]`:

| Config field | Details                                                   | Examples               |
| ------------ | --------------------------------------------------------- | ---------------------- |
| `rx`         | The colour of the RX (download) label and graph line      | `rx = "Red"`           |
| `tx`         | The colour of the TX (upload) label and graph line.       | `tx = "#ffffff"`       |
| `rx_total`   | The colour of the total RX (download) label in basic mode | `rx_total = "0, 0, 0"` |
| `tx_total`   | The colour of the total TX (upload) label in basic mode   | `tx_total = "#000"`    |

#### Battery

These can be set under `[styles.battery]`:

| Config field     | Details                                                                  | Examples                     |
| ---------------- | ------------------------------------------------------------------------ | ---------------------------- |
| `high_battery`   | The colour of the battery widget bar when the battery is over 50%        | `high_battery = "Red"`       |
| `medium_battery` | The colour of the battery widget bar when the battery between 10% to 50% | `medium_battery = "#ffffff"` |
| `low_battery`    | The colour of the battery widget bar when the battery is under 10%       | `low_battery = "0, 0, 0"`    |

#### Tables

These can be set under `[styles.tables]`:

| Config field | Details                        | Examples                                                       |
| ------------ | ------------------------------ | -------------------------------------------------------------- |
| `headers`    | Text styling for table headers | `headers = { color = "red", bg_color = "black", bold = true }` |

#### Graphs

These can be set under `[styles.graphs]`:

| Config field  | Details                                      | Examples                                                            |
| ------------- | -------------------------------------------- | ------------------------------------------------------------------- |
| `graph_color` | The general colour of the parts of the graph | `graph_color = "white"`                                             |
| `legend_text` | Text styling for graph's legend text         | `legend_text = { color = "black", bg_color = "blue", bold = true }` |

#### General widget settings

These can be set under `[styles.widgets]`:

| Config field      | Details                                                      | Examples                                                              |
| ----------------- | ------------------------------------------------------------ | --------------------------------------------------------------------- |
| `border`          | The colour of the widgets' borders                           | `border = "white"`                                                    |
| `selected_border` | The colour of a widget's borders when the widget is selected | `selected_border = "white"`                                           |
| `widget_title`    | Text styling for a widget's title                            | `widget_title = { color = "black", bg_color = "blue", bold = true }`  |
| `text`            | Text styling for text in general                             | `text = { color = "black", bg_color = "blue", bold = true }`          |
| `selected_text`   | Text styling for text when representing something selected   | `selected_text = { color = "black", bg_color = "blue", bold = true }` |
| `disabled_text`   | Text styling for text when representing something disabled   | `disabled_text = { color = "black", bg_color = "blue", bold = true }` |
