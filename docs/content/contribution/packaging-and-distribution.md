# Packaging and Distribution

Package maintainers are always welcome and appreciated! Here's some info on how one can help with package distribution and bottom.

## Pre-built binaries

The latest stable release can be found [here](https://github.com/ClementTsang/bottom/releases/latest), where you can find pre-built binaries in either a `tar.gz` or `zip` format.
Binaries here also include automatically generated shell completion files for zsh, bash, fish, and Powershell, which you may want to also install during the packaging
process.

You can also find a nightly build in the [releases page](https://github.com/ClementTsang/bottom/releases), built every day at 00:00 UTC off of the master branch.

## Building manually

If you want to manually build bottom rather than distributing a pre-built binary, you'll need the most recent version of stable Rust, which you can get with:

```bash
rustup update stable
```

You'll then want to build with:

```bash
cargo build --release --locked
```

Completion files are automatically generated during this process, and are located in the directory `target/release/build/bottom-<gibberish>/out`. Note there may be multiple folders that look like `target/release/build/bottom-<gibberish>`. To programmatically determine which is the right folder, you might want to use something like:

```bash
$(ls target/release/build/bottom-*/out/btm.bash | head -n1 | xargs dirname)
```

You may find the [Arch package install script template](https://github.com/ClementTsang/bottom/blob/master/deployment/linux/arch/PKGBUILD.template) useful as a reference.

## Adding an installation source

Once you've finished your installation source, if you want to mention it in the main bottom repo, fork the repo and add the installation method and any details to
the [`README.md`](https://github.com/ClementTsang/bottom/blob/master/README.md) file under the [Installation](https://github.com/ClementTsang/bottom#installation) section.
Once that's done, open a pull request - these will usually be approved of very quickly.

You can find more info on the contribution process [here](../issues-and-pull-requests/#pull-requests).
