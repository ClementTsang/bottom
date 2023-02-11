# Disk Widget

The disk widget provides a table of useful disk and partition information, like I/O per second and total usage.

<figure>
    <img src="../../../assets/screenshots/disk.webp" alt="A picture of an expanded disk widget."/>
</figure>

## Features

The disk widget provides the following information:

- Disk name
- Disk mount location
- Amount of space used
- Amount of space left
- Total amount of space
- Percentage of space used
- Read per second
- Write per second

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding            | Action                                                              |
| ------------------ | ------------------------------------------------------------------- |
| ++up++ , ++k++     | Move up within a widget                                             |
| ++down++ , ++j++   | Move down within a widget                                           |
| ++g+g++ , ++home++ | Jump to the first entry in the table                                |
| ++G++ , ++end++    | Jump to the last entry in the table                                 |
| ++d++              | Sort by disk, press again to reverse sorting order                  |
| ++m++              | Sort by mount, press again to reverse sorting order                 |
| ++u++              | Sort by amount used, press again to reverse sorting order           |
| ++n++              | Sort by amount free, press again to reverse sorting order           |
| ++t++              | Sort by total space available, press again to reverse sorting order |
| ++p++              | Sort by percentage used, press again to reverse sorting order       |
| ++r++              | Sort by read rate, press again to reverse sorting order             |
| ++w++              | Sort by write rate, press again to reverse sorting order            |

## Mouse bindings

| Binding     | Action                        |
| ----------- | ----------------------------- |
| ++lbutton++ | Selects an entry in the table |
