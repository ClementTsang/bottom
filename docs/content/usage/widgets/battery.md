# Battery Widget

!!! Warning

    The battery features are unavailable if the binary is compiled with the `battery` feature disabled or if there are no batteries on the system!

The battery widget provides information about batteries on the system.

<figure>
    <img src="../../../assets/screenshots/battery.webp" alt="A picture of an expanded battery widget."/>
</figure>

The battery widget can be enabled through either the `--battery` flag, the `battery = true` option in a config file, or specifying the widget in a custom layout.

## Features

The following data is displayed for batteries:

- Charge percent
- Consumption rate
- Charging state
- Time to empty/charge, based on the current state
- Battery health percent

The battery widget also supports devices with multiple batteries, and you can switch between them using the keyboard or the mouse.

## Key bindings

Note that key bindings are generally case-sensitive.

| Binding                               | Action                                                     |
| ------------------------------------- | ---------------------------------------------------------- |
| ++left++ <br/> ++h++ <br/> ++alt+h++  | Moves to the battery entry to the left of the current one  |
| ++right++ <br/> ++l++ <br/> ++alt+l++ | Moves to the battery entry to the right of the current one |

## Mouse bindings

| Binding     | Action                  |
| ----------- | ----------------------- |
| ++lbutton++ | Selects a battery entry |
