# Troubleshooting

## The graph points look broken/strange

It's possible that your graphs won't look great out of the box due to the reliance on braille fonts to draw them. One
example of this is seeing a bunch of missing font characters, caused when the terminal isn't configured properly to
render braille fonts.

<figure>
    <img src="../assets/screenshots/troubleshooting/no_braille.webp" alt="Example of a terminal with no braille font."/>
    <figcaption><sub>An example of missing braille fonts in Powershell</sub></figcaption>
</figure>

One alternative is to use the `--dot_marker` option to render graph charts using dots instead of the braille characters,
which generally seems better supported out of the box, at the expense of looking less intricate:

<figure>
    <img src="../assets/screenshots/troubleshooting/dots.webp" alt="Example of running bottom with the dot marker flag"/>
    <figcaption><sub>Example using <code>btm --dot_marker</code></sub></figcaption>
</figure>

Another (better) alternative is to install a font that supports braille fonts, and configure your terminal emulator to use it.
For example, installing something like [UBraille](https://yudit.org/download/fonts/UBraille/) or [Iosevka](https://github.com/be5invis/Iosevka)
and ensuring your terminal uses it should work.

### Braille font issues on Linux/macOS/Unix-like

Generally, the problem comes down to you either not having a font that supports the braille markers, or your terminal
emulator is not using the correct font for the braille markers.

See [here](https://github.com/cjbassi/gotop/issues/18) for possible fixes if you're having font issues on Linux, which
may also be helpful for macOS or other Unix-like systems.

If you're still having issues, feel free to open a [discussion](https://github.com/ClementTsang/bottom/discussions/new/)
question about it.

### Installing fonts for Windows Command Prompt/PowerShell

**Note: I would advise backing up your registry beforehand if you aren't sure what you are doing!**

Let's say you're installing [Iosevka](https://github.com/be5invis/Iosevka). The steps you can take are:

1. Install the font itself.
2. Open the registry editor, which you can do either by `Win+R` and opening `regedit`, or just opening it from the Start Menu.
3. In the registry editor, go to

   ```
   HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Console\TrueTypeFont
   ```

4. Here, add a new `String value`, and set the `Name` to a bunch of 0's (e.g. `000` - make sure the name isn't already used), then set the `Data` to the font name (e.g. `Iosevka`).

    <figure>
        <img src="../assets/screenshots/troubleshooting/regedit_fonts.webp" alt="Regedit menu showing how to add a new font for Command Prompt/PowerShell"/>
        <figcaption><sub>The last entry is the new entry for Iosevka</sub></figcaption>
    </figure>

5. Then, open the Command Prompt/PowerShell, and right-click on the top bar, and open "Properties":

    <figure>
        <img src="../assets/screenshots/troubleshooting/cmd_prompt_props.webp" alt="Opening the properties menu in Command Prompt/PowerShell"/>
    </figure>

6. From here, go to "Font", and set the font to your new font (so in this example, Iosevka):

<figure>
    <img src="../assets/screenshots/troubleshooting/cmd_prompt_font.webp" alt="Setting a new font in Command Prompt/PowerShell"/>
</figure>

## Why can't I see all my temperature sensors on Windows?

This is a [known limitation](./support/official.md#windows), some sensors may require admin privileges to get sensor data.

## Why don't I see dual batteries on Windows reported separately? (e.g. Thinkpads)

This is a [known limitation](./support/official.md#windows) which seems to be with how batteries are being detected on Windows.

## Why can't I see all my temperature sensors on WSL?

This is a [known limitation](./support/official.md#windows) with WSL. Due to how it works, hosts may not expose their
temperature sensors and therefore, temperature sensors might be missing.

## Why does WSL2 not match Task Manager?

This is a [known limitation](./support/official.md#windows) with WSL2. Due to how WSL2 works, the two might not match
up in terms of reported data.

## Why can't I see all my processes/process data on macOS?

This is a [known limitation](./support/official.md#macos), and you may have to run the program with elevated
privileges to work around it - for example:

```bash
sudo btm
```

**Please note that you should be certain that you trust any software you grant root privileges.**

There are measures taken to try to maximize the amount of information obtained without elevated privileges. For example,
one can modify the instructions found on the [htop wiki](https://github.com/hishamhm/htop/wiki/macOS:-run-without-sudo)
on how to run htop without sudo for bottom. However, **please** understand the potential security risks before doing so!

## My configuration file isn't working

If your configuration files aren't working, here are a few things to try:

### Check the formatting

It may be handy to refer to the automatically generated config files or the
[sample configuration files](https://github.com/ClementTsang/bottom/tree/main/sample_configs). The config files also
follow the [TOML](https://toml.io/en/) format.

Also make sure your config options are under the right table - for example, to set your temperature type, you must
set it under the `[flags]` table:

```toml
[flags]
temperature_type = "f"
```

Meanwhile, if you want to set a custom color scheme, it would be under the `[styles]` table:

```toml
[styles.tables.headers]
color="LightBlue"
```

To help validate your configuration files, there is [JSON Schema](https://json-schema.org/) support if your IDE/editor
supports it.

### Check the configuration file location

Make sure bottom is reading the right configuration file. By default, bottom looks for config files at these locations:

| OS      | Default Config Location                                                                                                                |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| macOS   | `$HOME/Library/Application Support/bottom/bottom.toml`<br/> `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml` |
| Linux   | `~/.config/bottom/bottom.toml` <br/> `$XDG_CONFIG_HOME/bottom/bottom.toml`                                                             |
| Windows | `C:\Users\<USER>\AppData\Roaming\bottom\bottom.toml`                                                                                   |

If you want to use a config file in another location, use the `--config` or `-C` flags along with the path to the configuration file, like so:

```bash
btm -C path_to_config
```

## My installation through snap has some widgets that are blank/show no data

Make sure bottom is given the correct permissions in order to collect data. [Snapcraft](https://snapcraft.io/docs/interface-management)
explains how to do so, but the TL;DR is:

```bash
sudo snap connect bottom:mount-observe
sudo snap connect bottom:hardware-observe
sudo snap connect bottom:system-observe
sudo snap connect bottom:process-control
```
