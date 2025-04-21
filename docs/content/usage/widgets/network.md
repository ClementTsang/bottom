# Network Widget

The network widget provides a visual representation of network input and output per second, as well as noting the total amount
received and transmitted.

<figure>
    <img src="../../../assets/screenshots/network/network.webp" alt="A picture of an expanded network widget."/>
</figure>

## Features

The legend displays the current reads and writes per second in bits, as well as the total amount read/written.

The y-axis automatically scales based on shown read/write values, and by default, is a linear scale based on base-10 units (e.x. kilobit, gigabit, etc.).
Through [configuration](../../configuration/command-line-options.md), the read/write per second unit can be changed to bytes, while the y-axis can be changed to a
log scale and/or use base-2 units (e.x. kibibit, gibibit, etc.).

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
