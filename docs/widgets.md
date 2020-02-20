# Widgets

## Compatibility

The compatibility of each widget and operating systems are, as of version 0.2.0, as follows:

| OS      | CPU | Memory | Disks | Temperature | Processes | Networks |
| ------- | --- | ------ | ----- | ----------- | --------- | -------- |
| Linux   | ✓   | ✓      | ✓     | ✓           | ✓         | ✓        |
| Windows | ✓   | ✓      | ✓     | ✗           | ✓         | ✓        |
| macOS   | ✓   | ✓      | ✓     | ✓           | ✓         | ✓        |

- Linux is tested on Arch Linux, using Kitty Terminal.

- Windows is tested on Windows 10, using Powershell.

- macOS is tested on macOS Catalina, using the base Terminal and Kitty Terminal.

## Widget information

### CPU

- Supports displaying specific cores (or average CPU usage if enabled); use `/` to allow for selection of cores to display, and `Space` to enable/disable them.

### Memory

- If no SWAP is available (size of 0) then no entry will show for SWAP.

### Disk

- I'm aware that Windows disk names are a bit strange... not sure if there's much I can do about it.

### Temperature

- Temperature sensors are sorted alphabetically and then by temperature (descending).

- Personally I found this to not work on Windows but YMMV.

### Network

- I'm aware that you cannot easily determine which graph line belongs to which entry unless you maximize - this is due to a limitation of tui-rs, and will be solved in a future release of the library.

- The graph is scaled logarithmically, by bytes, kibibytes, mebibytes, and gibibytes. I personally think this is enough for most people, but if you have a use case in which this isn't enough, let me know and I'll add in ways to increase it.

### Processes

- Filtering follows the convention of VS Code in terms of behaviour. For example, even in regex mode, it is not case sensitive if that is not enabled.
