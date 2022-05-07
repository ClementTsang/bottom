# Troubleshooting

## The graph points look broken/strange

It's possible that your graphs won't look great out of the box due to the reliance on braille fonts.

One example of this is seeing a bunch of missing font characters, caused when the terminal isn't configured properly to render braille fonts.

<figure>
    <img src="../assets/screenshots/troubleshooting/no_braille.webp" alt="Example of a terminal with no braille font."/>
    <figcaption>Powershell shown missing braille fonts</figcaption>
</figure>

Another example is when braille is rendered as a block of dots, with the non-coloured dots also appearing. This may look strange for some users, and it is also caused by misconfiguration or missing fonts.

<figure>
    <img src="../assets/screenshots/troubleshooting/weird_braille.webp" alt="Example of a terminal with the wrong braille font."/>
    <figcaption>Braille fonts rendering as a block of dots</figcaption>
</figure>

One alternative is to use the `--dot_marker` option to render graph charts using dots instead of the braille characters, which generally seems better supported out of the box,
at the expense of looking less intricate:

<figure>
    <img src="../assets/screenshots/troubleshooting/dots.webp" alt="Example of running bottom with the dot marker flag"/>
    <figcaption>Example using <code>btm --dot_marker</code></figcaption>
</figure>

Another (better) alternative is to install a font that supports braille fonts, and configure your terminal to use it. For example, installing
something like [UBraille](https://yudit.org/download/fonts/UBraille/) or [Iosevka](https://github.com/be5invis/Iosevka)
and ensuring your terminal uses it should work.

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
    <figcaption>The last entry is the new entry for Iosevka</code></figcaption>
    </figure>

5. Then, open the Command Prompt/PowerShell, and right click on the top bar, and open `Properties`:

    <figure>
    <img src="../assets/screenshots/troubleshooting/cmd_prompt_props.webp" alt="Opening the properties menu in Command Prompt/PowerShell"/>
    </figure>

6. From here, go to `Font`, and set the font to your new font (e.g. `Iosevka`):

    <figure>
    <img src="../assets/screenshots/troubleshooting/cmd_prompt_font.webp" alt="Setting a new font in Command Prompt/PowerShell"/>
    </figure>

## Why can't I see all my temperature sensors on Windows?

This is a known issue, and some sensors may require admin privileges to get sensor data.

## Why don't I see dual batteries on Windows reported separately? (e.g. Thinkpads)

This seems to be some issue with how batteries are being detected on Windows.

## Why can't I see all my temperature sensors on WSL?

Due to how WSL works, hosts may not expose their temperature sensors and therefore, temperature sensors might be missing.

## Why does WSL2 not match Task Manager?

Due to how WSL2 works, the two might not match up in terms of reported data.

## Why can't I see all my processes/process usage on macOS?

You may have to run the program with elevated privileges - for example:

```bash
sudo btm
```

_Please note that you should be certain that you trust any software you grant root privileges._

There are measures taken to try to maximize the amount of information obtained without elevated privileges, but there may still be some limitations.

## My configuration file isn't working

If your configuration files aren't working, here are a few things to try:

### Check the formatting

It may be handy to refer to the automatically generated config files or the [sample configuration files](https://github.com/ClementTsang/bottom/tree/master/sample_configs).
The config files also follow the [TOML](https://toml.io/en/) format.

Also make sure your config options are under the right table - for example, to set your temperature type, you must set it under the `[flags]` table:

```toml
[flags]
temperature_type = "f"
```

Meanwhile, if you want to set a custom color scheme, it would be under the `[colors]` table:

```toml
[colors]
table_header_color="LightBlue"
```

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

Make sure bottom is given the correct permissions. [Snapcraft](https://snapcraft.io/docs/interface-management) explains how to do so.
