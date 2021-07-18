# Memory Widget

The memory widget provides a visual representation of RAM and swap usage over time.

<figure>
    <img src="../../../assets/screenshots/memory.webp" alt="A picture of an expanded memory widget."/>
</figure>

## Features

The legend displays the current usage in terms of percentage and actual usage.
If the total RAM or swap available is 0, then it is automatically hidden from the legend and graph.

One can also adjust the displayed time range through either the keyboard or mouse, with a range of 30s to 600s.

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding   | Action                                  |
| --------- | --------------------------------------- |
| ++plus++  | Zoom in on chart (decrease time range)  |
| ++minus++ | Zoom out on chart (increase time range) |
| ++equal++ | Reset zoom                              |

## Mouse bindings

| Binding      | Action                                                         |
| ------------ | -------------------------------------------------------------- |
| ++"Scroll"++ | Scrolling up or down zooms in or out of the graph respectively |

## Calculations

Memory usage is calculated using the following formula:

```

```
