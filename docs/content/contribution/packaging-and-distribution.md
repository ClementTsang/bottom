# Packaging and Distribution

Package maintainers are always welcome and appreciated! Here's some info on how one can help with package distribution
and bottom.

## Pre-built binaries

The latest stable release can be found [here](https://github.com/ClementTsang/bottom/releases/latest), where you can
find pre-built binaries in either a `tar.gz` or `zip` format. Binaries here also include automatically generated shell
completion files for zsh, bash, fish, and Powershell, which you may want to also install during the packaging
process.

You can also find a nightly build in the [releases page](https://github.com/ClementTsang/bottom/releases), built every
day at 00:00 UTC off of the `main` branch.

In both cases, we use a combination of GitHub Actions and CirrusCI (mainly for FreeBSD and macOS M1) to create our
release binaries. [`build_releases.yml`](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/build_releases.yml)
contains the GitHub Action workflow used to do both of these, if reference is needed.

## Building manually

If you want to manually build bottom rather than distributing a pre-built binary, you'll need the most recent version
of stable Rust, which you can get with:

```bash
rustup update stable
```

You'll then want to build with:

```bash
cargo build --release --locked
```

### Manpage and completion generation

bottom uses a [`build.rs`](https://github.com/ClementTsang/bottom/blob/main/build.rs) script to automatically generate
a manpage and shell completions for the following shells:

- Bash
- Zsh
- Fish
- Powershell
- Elvish

If you want to generate manpages and/or completion files, set the `BTM_GENERATE` env var to a non-empty value. For
example, run something like this:

```bash
BTM_GENERATE=true cargo build --release --locked
```

This will automatically generate completion and manpage files in `target/tmp/bottom/`. If you wish to regenerate the
files, modify/delete either these files or set `BTM_GENERATE` to some other non-empty value to retrigger the build
script.

For more information, you may want to look at either the [`build.rs`](https://github.com/ClementTsang/bottom/blob/main/build.rs)
file or the [binary build CI workflow](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/build_releases.yml).

## Adding an installation source

Once you've finished your installation source, if you want to mention it in the main bottom repo, fork the repo and add
the installation method and any details to the [`README.md`](https://github.com/ClementTsang/bottom/blob/main/README.md)
file under the [Installation](https://github.com/ClementTsang/bottom#installation) section, as well as a corresponding
table of contents entry. Once that's done, open a pull request - these will usually be approved of very quickly.

You can find more info on the contribution process [here](issues-and-pull-requests.md#pull-requests).
