# Temperature Widget

The temperature widget provides a table of temperature sensors and their current temperature.

<figure>
    <img src="../../../assets/screenshots/temperature.webp" alt="A picture of an expanded temperature widget."/>
</figure>

## Features

The temperature widget provides the sensor name as well as its current temperature.

This widget can also be configured to display Nvidia GPU temperatures (`--disable_gpu` on Linux/Windows to disable).

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding            | Action                                                    |
| ------------------ | --------------------------------------------------------- |
| ++up++ , ++k++     | Move up within a widget                                   |
| ++down++ , ++j++   | Move down within a widget                                 |
| ++g+g++ , ++home++ | Jump to the first entry in the table                      |
| ++G++ , ++end++    | Jump to the last entry in the table                       |
| ++t++              | Sort by temperature, press again to reverse sorting order |
| ++s++              | Sort by sensor name, press again to reverse sorting order |

## Mouse bindings

| Binding     | Action                        |
| ----------- | ----------------------------- |
| ++lbutton++ | Selects an entry in the table |
