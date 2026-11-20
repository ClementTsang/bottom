# GPU Usage Graph Widget

The GPU usage graph widget displays the per-GPU utilization percentage over time.

<figure>
    <img src="../../../assets/screenshots/gpu_graph.webp" alt="A picture of an expanded GPU usage graph widget."/>
</figure>

## Features

The GPU usage graph shows the current GPU utilization for each GPU as a percentage.

The displayed time range can be adjusted through either the keyboard or mouse.

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

## How is GPU information gathered?

How we get GPU data depends on the platform:

- On Linux:
  - NVIDIA: Using the NVML crate
  - AMD/Intel: Manually parsing sysfs
- Other platforms use sysinfo's GPU API
