# Basic Mode

Basic mode is a special layout that removes all of the graphs and provides an interface that resembles (a very stripped-down version of) htop.

<figure>
    <img src="../../assets/screenshots/basic.webp" alt="A picture of bottom's basic mode."/>
</figure>

Basic mode can be enabled either through a command line flag:

```bash
btm -b

# or

btm --basic
```

or through the config:

```toml
[flags]
basic = true
```

## Notes

In this mode, widgets that use tables (temperatures, processes, disks, and batteries) are only shown one at a time.
One can switch between these widgets either by clicking the arrow buttons or by using the general widget selection shortcuts (for example, ++ctrl+left++ or ++H++)
to switch which widget is shown.

Also note that in this mode, widget expansion and custom layouts are disabled.

## Key bindings

Basic mode follows the same key bindings as normal, barring widget expansion being disabled, and that the ++"%"++ key while selecting the memory widget toggles between total usage and percentage.
