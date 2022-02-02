<div align="center">
  <h1>bottom (btm)</h1>

  <p>
  A customizable cross-platform graphical process/system monitor for the terminal.<br />Supports Linux, macOS, and Windows. Inspired by <a href=https://github.com/aksakalli/gtop>gtop</a>, <a href=https://github.com/xxxserxxx/gotop>gotop</a>, and <a href=https://github.com/htop-dev/htop/>htop</a>.
  </p>

[<img src="https://img.shields.io/github/workflow/status/ClementTsang/bottom/ci/master?style=flat-square&logo=github" alt="CI status">](https://github.com/ClementTsang/bottom/actions?query=branch%3Amaster)
[<img src="https://img.shields.io/codecov/c/github/ClementTsang/bottom?color=%23d9634d&logo=codecov&style=flat-square&token=IDD43CO0FP" alt="Codecov">](https://app.codecov.io/gh/ClementTsang/bottom)
[<img src="https://img.shields.io/crates/v/bottom.svg?style=flat-square" alt="crates.io link">](https://crates.io/crates/bottom)
[<img src="https://img.shields.io/badge/docs-nightly-88c0d0?style=flat-square&labelColor=555555&logoColor=white" alt="Nightly documentation">](https://clementtsang.github.io/bottom/nightly)
[<img src="https://img.shields.io/badge/docs-stable-66c2a5?style=flat-square&labelColor=555555&logoColor=white" alt="Stable documentation">](https://clementtsang.github.io/bottom/stable)

</div>

<img src="assets/demo.gif" alt="Quick demo recording showing off bottom's searching, expanding, and process killing."/>
<div align="center">
  <p>
    Demo GIF using the <a href="https://github.com/morhetz/gruvbox">Gruvbox</a> theme (<code>--color gruvbox</code>), along with <a href="https://www.ibm.com/plex/">IBM Plex Mono</a> and <a href="https://sw.kovidgoyal.net/kitty/">Kitty</a>
  </p>
</div>

## Features

As (yet another) process/system visualization and management application, bottom supports the typical features:

- Graphical visualization widgets for:

  - [CPU usage](https://clementtsang.github.io/bottom/nightly/usage/widgets/cpu/) over time, at an average and per-core level
  - [RAM and swap usage](https://clementtsang.github.io/bottom/nightly/usage/widgets/memory/) over time
  - [Network I/O usage](https://clementtsang.github.io/bottom/nightly/usage/widgets/network/) over time

  with support for zooming in/out the current time interval displayed.

- Widgets for displaying info about:

  - [Disk capacity/usage](https://clementtsang.github.io/bottom/nightly/usage/widgets/disk/)
  - [Temperature sensors](https://clementtsang.github.io/bottom/nightly/usage/widgets/temperature/)
  - [Battery usage](https://clementtsang.github.io/bottom/nightly/usage/widgets/battery/)

- [A process widget](https://clementtsang.github.io/bottom/nightly/usage/widgets/process/) for displaying, sorting, and searching info about processes, as well as support for:

  - Kill signals
  - Tree mode

- Cross-platform support for Linux, macOS, and Windows, with more planned in the future.

- [Customizable behaviour](https://clementtsang.github.io/bottom/nightly/configuration/command-line-flags/) that can be controlled with command-line flags or a config file, such as:

  - Custom and pre-built colour themes
  - Changing the default behaviour of some widgets
  - Changing the layout of widgets
  - Filtering out entries in disk and temperature widgets

- Some other nice stuff, like:

  - [An htop-inspired basic mode](https://clementtsang.github.io/bottom/nightly/usage/basic-mode/)
  - [Expansion, which focuses on just one widget](https://clementtsang.github.io/bottom/nightly/usage/general-usage/#expansion)

- And more!

You can find more details in [the documentation](https://clementtsang.github.io/bottom/nightly/usage/general-usage/).

## Support

bottom _officially_ supports the following operating systems and corresponding architectures:

- macOS (`x86_64`)
- Linux (`x86_64`, `i686`, `aarch64`)
- Windows (`x86_64`, `i686`)

These platforms are tested to work for the most part and issues on these platforms will be fixed if possible.
Furthermore, binaries are expected to be built and tested using the most recent version of stable Rust.

For more details on known problems and unsupported platforms, feel free to check out [the documentation page on support](https://clementtsang.github.io/bottom/nightly/support/official/).

## Installation

### Cargo

Installation via cargo is done by installing the `bottom` crate:

```bash
# If required, update Rust on the stable channel
rustup update stable

cargo install bottom

# Alternatively, --locked may be required due to how cargo install works
cargo install bottom --locked
```

### Arch Linux

There is an official package that can be installed with `pacman`:

```bash
sudo pacman -Syu bottom
```

### Debian/Ubuntu

A `.deb` file is provided on each [release](https://github.com/ClementTsang/bottom/releases/latest):

```bash
curl -LO https://github.com/ClementTsang/bottom/releases/download/0.6.8/bottom_0.6.8_amd64.deb
sudo dpkg -i bottom_0.6.8_amd64.deb
```

### Fedora/CentOS

Available in [COPR](https://copr.fedorainfracloud.org/coprs/atim/bottom/):

```bash
sudo dnf copr enable atim/bottom -y
sudo dnf install bottom
```

### Gentoo

Available in [GURU](https://wiki.gentoo.org/wiki/Project:GURU) and [dm9pZCAq](https://github.com/gentoo-mirror/dm9pZCAq) overlays:

```bash
sudo eselect repository enable guru
sudo emerge --sync guru
echo "sys-process/bottom" | sudo tee /etc/portage/package.accept_keywords/10-guru
sudo emerge sys-process/bottom::guru
```

or

```bash
sudo eselect repository enable dm9pZCAq
sudo emerge --sync dm9pZCAq
sudo emerge sys-process/bottom::dm9pZCAq
```

### Nix

```bash
nix-env -i bottom
```

### Solus

```
sudo eopkg it bottom
```

### Homebrew

```bash
brew install bottom
```

### MacPorts

```bash
sudo port selfupdate
sudo port install bottom
```

### Scoop

```bash
scoop install bottom
```

### Chocolatey

Choco package located [here](https://chocolatey.org/packages/bottom).
Since validation of the package takes time, it may take a while to become available after a release.

```bash
choco install bottom

# The version number may be required for newer releases during the approval process:
choco install bottom --version=0.6.8
```

### winget

You can find the packages [here](https://github.com/microsoft/winget-pkgs/tree/master/manifests/Clement/bottom).
Since validation of the package takes time, it may take a while to become available after a release.

```bash
winget install bottom

# Alternatively
winget install Clement.bottom
```

You can also manually do the same thing by going to the [latest release](https://github.com/ClementTsang/bottom/releases/latest)
and installing via the `.msi` file.

You can uninstall via Control Panel, Options, or `winget --uninstall bottom`.

### Manually

There are a few ways to go about doing this manually. Note that you probably want
to do so using the most recent version of stable Rust, which is how the binaries are built:

```bash
# If required, update Rust on the stable channel first
rustup update stable

# Option 1 - Download from releases and install
curl -LO https://github.com/ClementTsang/bottom/archive/0.6.8.tar.gz
tar -xzvf 0.6.8.tar.gz
cargo install --path .

# Option 2 - Clone from master and install manually
git clone https://github.com/ClementTsang/bottom
cd bottom
cargo install --path .

# Option 3 - Clone and install directly from the repo all via Cargo
cargo install --git https://github.com/ClementTsang/bottom
```

### Binaries

You can also try to use the generated release binaries and manually install on your system:

- [Latest stable release](https://github.com/ClementTsang/bottom/releases/latest), generated off of the release branch
- [Latest nightly release](https://github.com/ClementTsang/bottom/releases/tag/nightly), generated daily off of the master branch at 00:00 UTC

#### Auto-completion

The release binaries are packaged with shell auto-completion files for bash, fish, zsh, and Powershell. To install them:

- For bash, move `btm.bash` to `$XDG_CONFIG_HOME/bash_completion or /etc/bash_completion.d/`.
- For fish, move `btm.fish` to `$HOME/.config/fish/completions/`.
- For zsh, move `_btm` to one of your `$fpath` directories.
- For PowerShell, add `. _btm.ps1` to your PowerShell
  [profile](<https://docs.microsoft.com/en-us/previous-versions//bb613488(v=vs.85)>).

## Usage

You can run bottom using `btm`.

- For help on flags, use `btm -h` for a quick overview or `btm --help` for more details.
- For info on key and mouse bindings, press `?` inside bottom or refer to the [documentation](https://clementtsang.github.io/bottom/nightly/).

You can find more information on usage in the [documentation](https://clementtsang.github.io/bottom/nightly/).

## Configuration

bottom accepts a number of command-line arguments to change the behaviour of the application as desired. Additionally, bottom will automatically
generate a configuration file on the first launch, which one can change as appropriate.

More details on configuration can be found [in the documentation](https://clementtsang.github.io/bottom/nightly/configuration/config-file/default-config/).

## Contribution

Whether it's reporting bugs, suggesting features, maintaining packages, or submitting a PR,
contribution is always welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for details on how to
contribute to bottom.

### Contributors

Thanks to all contributors:

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="http://shilangyu.github.io"><img src="https://avatars3.githubusercontent.com/u/29288116?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Marcin Wojnarowski</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=shilangyu" title="Code">💻</a> <a href="#platform-shilangyu" title="Packaging/porting to new platform">📦</a></td>
    <td align="center"><a href="http://neosmart.net/"><img src="https://avatars3.githubusercontent.com/u/606923?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Mahmoud Al-Qudsi</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=mqudsi" title="Code">💻</a></td>
    <td align="center"><a href="https://andys8.de"><img src="https://avatars0.githubusercontent.com/u/13085980?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Andy</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=andys8" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/HarHarLinks"><img src="https://avatars0.githubusercontent.com/u/2803622?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Kim Brose</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=HarHarLinks" title="Code">💻</a></td>
    <td align="center"><a href="https://svenstaro.org"><img src="https://avatars0.githubusercontent.com/u/1664?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Sven-Hendrik Haase</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=svenstaro" title="Documentation">📖</a></td>
    <td align="center"><a href="https://liberapay.com/Artem4/"><img src="https://avatars0.githubusercontent.com/u/5614476?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Artem Polishchuk</b></sub></a><br /><a href="#platform-tim77" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/ClementTsang/bottom/commits?author=tim77" title="Documentation">📖</a></td>
    <td align="center"><a href="http://ruby-journal.com/"><img src="https://avatars2.githubusercontent.com/u/135605?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Trung Lê</b></sub></a><br /><a href="#platform-runlevel5" title="Packaging/porting to new platform">📦</a> <a href="#infra-runlevel5" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://github.com/dm9pZCAq"><img src="https://avatars1.githubusercontent.com/u/46228973?v=4?s=100" width="100px;" alt=""/><br /><sub><b>dm9pZCAq</b></sub></a><br /><a href="#platform-dm9pZCAq" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/ClementTsang/bottom/commits?author=dm9pZCAq" title="Documentation">📖</a></td>
    <td align="center"><a href="https://lukor.org"><img src="https://avatars2.githubusercontent.com/u/10536802?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Lukas Rysavy</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=LlinksRechts" title="Code">💻</a></td>
    <td align="center"><a href="http://hamberg.no/erlend"><img src="https://avatars3.githubusercontent.com/u/16063?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Erlend Hamberg</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=ehamberg" title="Code">💻</a></td>
    <td align="center"><a href="https://onee3.org"><img src="https://avatars.githubusercontent.com/u/4507647?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Frederick Zhang</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=Frederick888" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/pvanheus"><img src="https://avatars.githubusercontent.com/u/4154788?v=4?s=100" width="100px;" alt=""/><br /><sub><b>pvanheus</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=pvanheus" title="Code">💻</a></td>
    <td align="center"><a href="https://zebulon.dev/"><img src="https://avatars.githubusercontent.com/u/14242997?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Zeb Piasecki</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=vlakreeh" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/georgybog"><img src="https://avatars.githubusercontent.com/u/60893791?v=4?s=100" width="100px;" alt=""/><br /><sub><b>georgybog</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=georgybog" title="Documentation">📖</a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://github.com/briandipalma"><img src="https://avatars.githubusercontent.com/u/1597820?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Brian Di Palma</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=briandipalma" title="Documentation">📖</a></td>
    <td align="center"><a href="https://dakyskye.github.io"><img src="https://avatars.githubusercontent.com/u/32128756?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Lasha Kanteladze</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=dakyskye" title="Documentation">📖</a></td>
    <td align="center"><a href="https://github.com/herbygillot"><img src="https://avatars.githubusercontent.com/u/618376?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Herby Gillot</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=herbygillot" title="Documentation">📖</a></td>
    <td align="center"><a href="https://github.com/yellowsquid"><img src="https://avatars.githubusercontent.com/u/46519298?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Greg Brown</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=yellowsquid" title="Code">💻</a></td>
    <td align="center"><a href="https://github.com/TotalCaesar659"><img src="https://avatars.githubusercontent.com/u/14265316?v=4?s=100" width="100px;" alt=""/><br /><sub><b>TotalCaesar659</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=TotalCaesar659" title="Documentation">📖</a></td>
    <td align="center"><a href="https://github.com/grawlinson"><img src="https://avatars.githubusercontent.com/u/4408051?v=4?s=100" width="100px;" alt=""/><br /><sub><b>George Rawlinson</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=grawlinson" title="Documentation">📖</a> <a href="#platform-grawlinson" title="Packaging/porting to new platform">📦</a></td>
    <td align="center"><a href="https://www.frogorbits.com/"><img src="https://avatars.githubusercontent.com/u/101246?v=4?s=100" width="100px;" alt=""/><br /><sub><b>adiabatic</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=adiabatic" title="Documentation">📖</a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://electronsweatshop.com"><img src="https://avatars.githubusercontent.com/u/354506?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Randy Barlow</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=bowlofeggs" title="Code">💻</a></td>
    <td align="center"><a href="http://jackson.dev"><img src="https://avatars.githubusercontent.com/u/160646?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Patrick Jackson</b></sub></a><br /><a href="#ideas-patricksjackson" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/ClementTsang/bottom/commits?author=patricksjackson" title="Documentation">📖</a></td>
    <td align="center"><a href="https://github.com/mati865"><img src="https://avatars.githubusercontent.com/u/1174646?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Mateusz Mikuła</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=mati865" title="Code">💻</a></td>
    <td align="center"><a href="https://blog.guillaume-gomez.fr"><img src="https://avatars.githubusercontent.com/u/3050060?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Guillaume Gomez</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=GuillaumeGomez" title="Code">💻</a></td>
  </tr>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## Thanks

- This project is very much inspired by [gotop](https://github.com/xxxserxxx/gotop),
  [gtop](https://github.com/aksakalli/gtop), and [htop](https://github.com/htop-dev/htop/).

- This application was written with many, _many_ libraries, and built on the
  work of many talented people. This application would be impossible without their
  work. I used to thank them all individually but the list got too large...

- And of course, another round of thanks to all contributors and package maintainers!
