# Build Process

!!! Warning

    This section is currently somewhat WIP.

!!! Warning

    This section is intended for people who wish to work on/build/distribute bottom, not general users.

## Overview

bottom manages its own binary builds for nightly and stable release purposes. The core build workflow is handled by [`build_releases.yml`](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/build_releases.yml), called by a wrapper workflow for [nightly](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/nightly.yml) and [stable](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/deployment.yml) releases. Builds take place via GitHub Actions.

The main things built are:

- Binaries for various platforms
- MSI installer for Windows
- `.deb` package for Debian and its derivatives

This documentation gives a high-level overview of the build process for each part. For the most up-to-date and detailed reference, definitely refer back to the [`build_releases.yml`](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/build_releases.yml) file.

## Binaries

Binaries are built currently for various targets. Note that not all of these are officially supported. The following general steps are performed:

- Set up the Rust toolchain for the action runner.
- Enable cache.
- Build a release build with:

  - `--features deploy`, which enables only crates needed for release builds.
  - `--locked` to lock the dependency versions.
  - The following env variables set:

    - `BTM_GENERATE: true`
    - `COMPLETION_DIR: "target/tmp/bottom/completion/"`
    - `MANPAGE_DIR: "target/tmp/bottom/manpage/"`

    These generate the manpages and shell completions (see [Packaging](../packaging-and-distribution.md) for some more information).

- Bundle the binaries and manpage/completions.
- Cleanup.

Some builds use [`cross`](https://github.com/cross-rs/cross) to do cross-compilation builds for architectures otherwise not natively supported by the runner.

## MSI

This builds a full Windows installer using [`cargo-wix`](https://github.com/volks73/cargo-wix). This requires some setup beforehand with some dependencies:

- Net-Framework-Core (handled by Powershell)
- wixtoolset (handled by chocolatey)
- Rust toolchain

After that, cache is enabled, and `cargo wix` takes care of the rest.

## `.deb`

Currently, `.deb` files are built for x86 and ARM architectures (`armv7`, `aarch64`). This is handled by [`cargo-deb`](https://crates.io/crates/cargo-deb).

- For x86, this is handled natively with just `cargo-deb`.
- For ARM, this uses a Docker container, [cargo-deb-arm](https://github.com/ClementTsang/cargo-deb-arm), which correctly sets the dependencies and architecture for the generated `.deb` file.

There are additional checks via `dpkg` to ensure the architecture is correctly set.
