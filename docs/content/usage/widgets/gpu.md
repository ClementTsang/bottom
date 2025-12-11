# GPU Widget

The GPU widget displays a visual representation of GPU usage over a time range.

## Supported GPUs

- **NVIDIA**
- **AMD**
- **Intel** (Linux only)

## Features

The GPU widget is composed of two parts: the graph and the legend:

- The graph displays the usage data for the currently selected entry as a percentage
- The legend displays the GPU name, load percentage, VRAM usage, and temperature

Users can scroll through the legend using either the keyboard or mouse to select which entry to display on the graph.

One can also adjust the displayed time range through either the keyboard or mouse, with a range of 30s to 600s.

## Key bindings

Note that key bindings are generally case-sensitive.

### Graph

| Binding   | Action                                  |
| --------- | --------------------------------------- |
| ++plus++  | Zoom in on chart (decrease time range)  |
| ++minus++ | Zoom out on chart (increase time range) |
| ++equal++ | Reset zoom                              |

### Legend

| Binding            | Action                                |
| ------------------ | ------------------------------------- |
| ++up++ , ++k++     | Move up within a widget               |
| ++down++ , ++j++   | Move down within a widget             |
| ++g+g++ , ++home++ | Jump to the first entry in the legend |
| ++G++ , ++end++    | Jump to the last entry in the legend  |

## Mouse bindings

### Graph

| Binding      | Action                                                         |
| ------------ | -------------------------------------------------------------- |
| ++"Scroll"++ | Scrolling up or down zooms in or out of the graph respectively |

### Legend

| Binding      | Action                                            |
| ------------ | ------------------------------------------------- |
| ++"Scroll"++ | Scroll through options to display in the graph    |
| ++lbutton++  | Selects a GPU to show in the graph                |
