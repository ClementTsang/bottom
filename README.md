<div align="center">
  <h1>bottom</h1>

  <p>
  A customizable cross-platform graphical process/system monitor for the terminal.<br />Supports Linux, macOS, and Windows. Inspired by <a href=https://github.com/aksakalli/gtop>gtop</a>, <a href=https://github.com/xxxserxxx/gotop>gotop</a>, and <a href=https://github.com/htop-dev/htop/>htop</a>.
  </p>

[<img src="https://img.shields.io/github/actions/workflow/status/ClementTsang/bottom/ci.yml?branch=master&style=flat-square&logo=github" alt="CI status">](https://github.com/ClementTsang/bottom/actions?query=branch%3Amaster)
[<img src="https://img.shields.io/crates/v/bottom.svg?style=flat-square" alt="crates.io link">](https://crates.io/crates/bottom)
[<img src="https://img.shields.io/badge/docs-stable-66c2a5?style=flat-square&labelColor=555555&logoColor=white" alt="Stable documentation">](https://clementtsang.github.io/bottom/stable)
[<img src="https://img.shields.io/badge/docs-nightly-88c0d0?style=flat-square&labelColor=555555&logoColor=white" alt="Nightly documentation">](https://clementtsang.github.io/bottom/nightly)

</div>

<div align="center">
  <img src="assets/demo.gif" alt="Quick demo recording showing off bottom's searching, expanding, and process killing."/>
  <p>
    <sub>
      Demo GIF using the <a href="https://github.com/morhetz/gruvbox">Gruvbox</a> theme (<code>--color gruvbox</code>), along with <a href="https://www.ibm.com/plex/">IBM Plex Mono</a> and <a href="https://sw.kovidgoyal.net/kitty/">Kitty</a>
    </sub>
  </p>
</div>

## Table of contents <!-- omit in toc -->

- [Features](#features)
- [Support](#support)
  - [Official](#official)
  - [Unofficial](#unofficial)
- [Installation](#installation)
  - [Cargo](#cargo)
  - [Arch Linux](#arch-linux)
  - [Debian/Ubuntu](#debianubuntu)
  - [Snap](#snap)
  - [Fedora/CentOS](#fedoracentos)
  - [Gentoo](#gentoo)
  - [Nix](#nix)
  - [Solus](#solus)
  - [Void](#void)
  - [Homebrew](#homebrew)
  - [MacPorts](#macports)
  - [Scoop](#scoop)
  - [Chocolatey](#chocolatey)
  - [winget](#winget)
  - [Windows installer](#windows-installer)
  - [Manual installation](#manual-installation)
  - [Binaries](#binaries)
    - [Auto-completion](#auto-completion)
- [Usage](#usage)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Contribution](#contribution)
  - [Contributors](#contributors)
- [Thanks](#thanks)

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

### Official

bottom _officially_ supports the following operating systems and corresponding architectures:

- macOS (`x86_64`)
- Linux (`x86_64`, `i686`, `aarch64`)
- Windows (`x86_64`, `i686`)

These platforms are tested to work for the most part and issues on these platforms will be fixed if possible.
Furthermore, binaries are expected to be built and tested using the most recent version of stable Rust.

For more details on supported platforms and known problems, check out [the documentation](https://clementtsang.github.io/bottom/nightly/support/official/).

### Unofficial

bottom may work on a number of platforms that aren't officially supported. Note that unsupported platforms:

- Might not be tested in CI to build or pass tests (see [here](./.github/workflows/ci.yml) for checked platforms).
- Might not be properly tested by maintainers prior to a stable release.
- May only receive limited support, such as missing features or bugs that may not be fixed.

Note that some unsupported platforms may eventually be officially supported (e.g., FreeBSD, M1 macOS devices).

A non-comprehensive list of some currently unofficially supported platforms that may compile/work include:

- FreeBSD on `x86_64`
- Linux on `armv6`, `armv7`, `powerpc64le`, `riscv64gc`
- macOS on `aarch64`

For more details on unsupported platforms and known problems, check out [the documentation](https://clementtsang.github.io/bottom/nightly/support/unofficial/).

## Installation

### Cargo

Installation via cargo is done by installing the `bottom` crate:

```bash
# If required, update Rust on the stable channel
rustup update stable

cargo install bottom --locked

# Alternatively, --locked may be omitted if you wish to not used locked dependencies:
cargo install bottom
```

### Arch Linux

There is an [official package](https://archlinux.org/packages/community/x86_64/bottom/) that can be installed with `pacman`:

```bash
sudo pacman -S bottom
```

### Debian/Ubuntu

A `.deb` file is provided on each [stable release](https://github.com/ClementTsang/bottom/releases/latest) and
[nightly builds](https://github.com/ClementTsang/bottom/releases/tag/nightly) for x86, aarch64, and armv7
(note stable ARM builds are only available for 0.6.8 and later). If you want to install this way, do something like:

```bash
# x86-64
curl -LO https://github.com/ClementTsang/bottom/releases/download/0.8.0/bottom_0.8.0_amd64.deb
sudo dpkg -i bottom_0.8.0_amd64.deb

# ARM64
curl -LO https://github.com/ClementTsang/bottom/releases/download/0.8.0/bottom_0.8.0_arm64.deb
sudo dpkg -i bottom_0.8.0_arm64.deb

# ARM
curl -LO https://github.com/ClementTsang/bottom/releases/download/0.8.0/bottom_0.8.0_armhf.deb
sudo dpkg -i bottom_0.8.0_armhf.deb
```

### Snap

bottom is available as a [snap](https://snapcraft.io/install/bottom/ubuntu):

```bash
sudo snap install bottom

# To allow the program to run as intended
sudo snap connect bottom:mount-observe
sudo snap connect bottom:hardware-observe
sudo snap connect bottom:system-observe
sudo snap connect bottom:process-control
```

### Fedora/CentOS

Available in [COPR](https://copr.fedorainfracloud.org/coprs/atim/bottom/):

```bash
sudo dnf copr enable atim/bottom -y
sudo dnf install bottom
```

### Gentoo

Available in the official Gentoo repo:

```bash
sudo emerge --ask sys-process/bottom
```

### Nix

```bash
nix-env -i bottom
```

### Solus

```bash
sudo eopkg it bottom
```

### Void

```bash
sudo xbps-install bottom
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

Chocolatey packages are located [here](https://chocolatey.org/packages/bottom).

```bash
choco install bottom

# The version number may need to be specified for newer releases for the first
# few hours/days during the approval process:
choco install bottom --version=0.8.0
```

### winget

The winget package can be found [here](https://github.com/microsoft/winget-pkgs/tree/master/manifests/Clement/bottom):

```bash
winget install bottom

# Alternatively, to be more specific:
winget install Clement.bottom
```

You can uninstall via Control Panel, Options, or `winget --uninstall bottom`.

### Windows installer

You can also manually install bottom as a Windows program by going to the [latest release](https://github.com/ClementTsang/bottom/releases/latest)
and installing via the `.msi` file.

### Manual installation

There are a few ways to go about doing this manually. Note that you probably want
to do so using the most recent version of stable Rust, which is how the binaries are built:

```bash
# If required, update Rust on the stable channel first
rustup update stable

# Option 1 - Download from releases and install
curl -LO https://github.com/ClementTsang/bottom/archive/0.8.0.tar.gz
tar -xzvf 0.8.0.tar.gz
cargo install --path .

# Option 2 - Clone from master and install manually
git clone https://github.com/ClementTsang/bottom
cd bottom
cargo install --path .

# Option 3 - Clone and install directly from the repo all via Cargo
cargo install --git https://github.com/ClementTsang/bottom

# You can also pass in the target-cpu=native flag for
# better CPU-specific optimizations. For example:
RUSTFLAGS="-C target-cpu=native" cargo install --path .
```

### Binaries

You can also try to use the generated release binaries and manually install on your system:

- [Latest stable release](https://github.com/ClementTsang/bottom/releases/latest), generated off of the release branch
- [Latest nightly release](https://github.com/ClementTsang/bottom/releases/tag/nightly), generated daily off of the master branch at 00:00 UTC
  - Note that for now, FreeBSD and ARM macOS builds are primarily only available on the nightly release.

#### Auto-completion

The release binaries are packaged with shell auto-completion files for bash, fish, zsh, and Powershell. To install them:

- For bash, move `btm.bash` to `$XDG_CONFIG_HOME/bash_completion or /etc/bash_completion.d/`.
- For fish, move `btm.fish` to `$HOME/.config/fish/completions/`.
- For zsh, move `_btm` to one of your `$fpath` directories.
- For PowerShell, add `. _btm.ps1` to your PowerShell
  [profile](<https://docs.microsoft.com/en-us/previous-versions//bb613488(v=vs.85)>).

The individual auto-completion files are also included in the stable/nightly releases as `completion.tar.gz`.

## Usage

You can run bottom using `btm`.

- For help on flags, use `btm -h` for a quick overview or `btm --help` for more details.
- For info on key and mouse bindings, press `?` inside bottom or refer to the [documentation](https://clementtsang.github.io/bottom/nightly/).

You can find more information on usage in the [documentation](https://clementtsang.github.io/bottom/nightly/).

## Configuration

bottom accepts a number of command-line arguments to change the behaviour of the application as desired. Additionally, bottom will automatically
generate a configuration file on the first launch, which one can change as appropriate.

More details on configuration can be found [in the documentation](https://clementtsang.github.io/bottom/nightly/configuration/config-file/default-config/).

## Troubleshooting

If some things aren't working, give the [Troubleshooting page](https://clementtsang.github.io/bottom/nightly/troubleshooting) a look. If things still aren't
working, then consider opening [a question](https://github.com/ClementTsang/bottom/discussions) or filing a [bug report](https://github.com/ClementTsang/bottom/issues/new/choose).

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
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="http://shilangyu.github.io"><img src="https://avatars3.githubusercontent.com/u/29288116?v=4?s=100" width="100px;" alt="Marcin Wojnarowski"/><br /><sub><b>Marcin Wojnarowski</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=shilangyu" title="Code">💻</a> <a href="#platform-shilangyu" title="Packaging/porting to new platform">📦</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://neosmart.net/"><img src="https://avatars3.githubusercontent.com/u/606923?v=4?s=100" width="100px;" alt="Mahmoud Al-Qudsi"/><br /><sub><b>Mahmoud Al-Qudsi</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=mqudsi" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://andys8.de"><img src="https://avatars0.githubusercontent.com/u/13085980?v=4?s=100" width="100px;" alt="Andy"/><br /><sub><b>Andy</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=andys8" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/HarHarLinks"><img src="https://avatars0.githubusercontent.com/u/2803622?v=4?s=100" width="100px;" alt="Kim Brose"/><br /><sub><b>Kim Brose</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=HarHarLinks" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://svenstaro.org"><img src="https://avatars0.githubusercontent.com/u/1664?v=4?s=100" width="100px;" alt="Sven-Hendrik Haase"/><br /><sub><b>Sven-Hendrik Haase</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=svenstaro" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://liberapay.com/Artem4/"><img src="https://avatars0.githubusercontent.com/u/5614476?v=4?s=100" width="100px;" alt="Artem Polishchuk"/><br /><sub><b>Artem Polishchuk</b></sub></a><br /><a href="#platform-tim77" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/ClementTsang/bottom/commits?author=tim77" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://ruby-journal.com/"><img src="https://avatars2.githubusercontent.com/u/135605?v=4?s=100" width="100px;" alt="Trung Lê"/><br /><sub><b>Trung Lê</b></sub></a><br /><a href="#platform-runlevel5" title="Packaging/porting to new platform">📦</a> <a href="#infra-runlevel5" title="Infrastructure (Hosting, Build-Tools, etc)">🚇</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/dm9pZCAq"><img src="https://avatars1.githubusercontent.com/u/46228973?v=4?s=100" width="100px;" alt="dm9pZCAq"/><br /><sub><b>dm9pZCAq</b></sub></a><br /><a href="#platform-dm9pZCAq" title="Packaging/porting to new platform">📦</a> <a href="https://github.com/ClementTsang/bottom/commits?author=dm9pZCAq" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://lukor.org"><img src="https://avatars2.githubusercontent.com/u/10536802?v=4?s=100" width="100px;" alt="Lukas Rysavy"/><br /><sub><b>Lukas Rysavy</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=LlinksRechts" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://hamberg.no/erlend"><img src="https://avatars3.githubusercontent.com/u/16063?v=4?s=100" width="100px;" alt="Erlend Hamberg"/><br /><sub><b>Erlend Hamberg</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=ehamberg" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://onee3.org"><img src="https://avatars.githubusercontent.com/u/4507647?v=4?s=100" width="100px;" alt="Frederick Zhang"/><br /><sub><b>Frederick Zhang</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=Frederick888" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/pvanheus"><img src="https://avatars.githubusercontent.com/u/4154788?v=4?s=100" width="100px;" alt="pvanheus"/><br /><sub><b>pvanheus</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=pvanheus" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://zebulon.dev/"><img src="https://avatars.githubusercontent.com/u/14242997?v=4?s=100" width="100px;" alt="Zeb Piasecki"/><br /><sub><b>Zeb Piasecki</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=vlakreeh" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/briandipalma"><img src="https://avatars.githubusercontent.com/u/1597820?v=4?s=100" width="100px;" alt="Brian Di Palma"/><br /><sub><b>Brian Di Palma</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=briandipalma" title="Documentation">📖</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://dakyskye.github.io"><img src="https://avatars.githubusercontent.com/u/32128756?v=4?s=100" width="100px;" alt="Lasha Kanteladze"/><br /><sub><b>Lasha Kanteladze</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=dakyskye" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/herbygillot"><img src="https://avatars.githubusercontent.com/u/618376?v=4?s=100" width="100px;" alt="Herby Gillot"/><br /><sub><b>Herby Gillot</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=herbygillot" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/yellowsquid"><img src="https://avatars.githubusercontent.com/u/46519298?v=4?s=100" width="100px;" alt="Greg Brown"/><br /><sub><b>Greg Brown</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=yellowsquid" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/TotalCaesar659"><img src="https://avatars.githubusercontent.com/u/14265316?v=4?s=100" width="100px;" alt="TotalCaesar659"/><br /><sub><b>TotalCaesar659</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=TotalCaesar659" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/grawlinson"><img src="https://avatars.githubusercontent.com/u/4408051?v=4?s=100" width="100px;" alt="George Rawlinson"/><br /><sub><b>George Rawlinson</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=grawlinson" title="Documentation">📖</a> <a href="#platform-grawlinson" title="Packaging/porting to new platform">📦</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.frogorbits.com/"><img src="https://avatars.githubusercontent.com/u/101246?v=4?s=100" width="100px;" alt="adiabatic"/><br /><sub><b>adiabatic</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=adiabatic" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://electronsweatshop.com"><img src="https://avatars.githubusercontent.com/u/354506?v=4?s=100" width="100px;" alt="Randy Barlow"/><br /><sub><b>Randy Barlow</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=bowlofeggs" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="http://jackson.dev"><img src="https://avatars.githubusercontent.com/u/160646?v=4?s=100" width="100px;" alt="Patrick Jackson"/><br /><sub><b>Patrick Jackson</b></sub></a><br /><a href="#ideas-patricksjackson" title="Ideas, Planning, & Feedback">🤔</a> <a href="https://github.com/ClementTsang/bottom/commits?author=patricksjackson" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/mati865"><img src="https://avatars.githubusercontent.com/u/1174646?v=4?s=100" width="100px;" alt="Mateusz Mikuła"/><br /><sub><b>Mateusz Mikuła</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=mati865" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://blog.guillaume-gomez.fr"><img src="https://avatars.githubusercontent.com/u/3050060?v=4?s=100" width="100px;" alt="Guillaume Gomez"/><br /><sub><b>Guillaume Gomez</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=GuillaumeGomez" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/shurizzle"><img src="https://avatars.githubusercontent.com/u/203655?v=4?s=100" width="100px;" alt="shura"/><br /><sub><b>shura</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=shurizzle" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.wezm.net/"><img src="https://avatars.githubusercontent.com/u/21787?v=4?s=100" width="100px;" alt="Wesley Moore"/><br /><sub><b>Wesley Moore</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=wezm" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/xgdgsc"><img src="https://avatars.githubusercontent.com/u/1189869?v=4?s=100" width="100px;" alt="xgdgsc"/><br /><sub><b>xgdgsc</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=xgdgsc" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/ViridiCanis"><img src="https://avatars.githubusercontent.com/u/49595344?v=4?s=100" width="100px;" alt="ViridiCanis"/><br /><sub><b>ViridiCanis</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=ViridiCanis" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/jamartin9"><img src="https://avatars.githubusercontent.com/u/7027701?v=4?s=100" width="100px;" alt="Justin Martin"/><br /><sub><b>Justin Martin</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=jamartin9" title="Code">💻</a> <a href="https://github.com/ClementTsang/bottom/commits?author=jamartin9" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/DianaNites"><img src="https://avatars.githubusercontent.com/u/5275194?v=4?s=100" width="100px;" alt="Diana"/><br /><sub><b>Diana</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=DianaNites" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://hervyqa.id"><img src="https://avatars.githubusercontent.com/u/45872139?v=4?s=100" width="100px;" alt="Hervy Qurrotul Ainur Rozi"/><br /><sub><b>Hervy Qurrotul Ainur Rozi</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=hervyqa" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://mrivnak.github.io"><img src="https://avatars.githubusercontent.com/u/7389355?v=4?s=100" width="100px;" alt="Mike Rivnak"/><br /><sub><b>Mike Rivnak</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=mrivnak" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lroobrou"><img src="https://avatars.githubusercontent.com/u/35152113?v=4?s=100" width="100px;" alt="lroobrou"/><br /><sub><b>lroobrou</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=lroobrou" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://cube64128.xyz/"><img src="https://avatars.githubusercontent.com/u/18757988?v=4?s=100" width="100px;" alt="database64128"/><br /><sub><b>database64128</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=database64128" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sou-chon"><img src="https://avatars.githubusercontent.com/u/35537528?v=4?s=100" width="100px;" alt="Chon Sou"/><br /><sub><b>Chon Sou</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=sou-chon" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Drsheppard01"><img src="https://avatars.githubusercontent.com/u/60893791?v=4?s=100" width="100px;" alt="DrSheppard"/><br /><sub><b>DrSheppard</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=Drsheppard01" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/RaresCon"><img src="https://avatars.githubusercontent.com/u/95525840?v=4?s=100" width="100px;" alt="Rareș Constantin"/><br /><sub><b>Rareș Constantin</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=RaresCon" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://felipesuri.com"><img src="https://avatars.githubusercontent.com/u/50281523?v=4?s=100" width="100px;" alt="felipesuri"/><br /><sub><b>felipesuri</b></sub></a><br /><a href="https://github.com/ClementTsang/bottom/commits?author=felipesuri" title="Documentation">📖</a></td>
    </tr>
  </tbody>
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
