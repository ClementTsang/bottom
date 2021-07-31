# Support

## Official support

bottom _officially_ supports the following operating systems and corresponding architectures:

- macOS (`x86_64`)
- Linux (`x86_64`, `i686`, `aarch64`)
- Windows (`x86_64`, `i686`)

These platforms are tested to work (with caveats, see below) and issues on these platforms will be fixed if possible.

Furthermore, binaries are expected to be built and tested using the most recent version of stable Rust - if you are manually building
bottom from the repo/source, then please try that as well.

### Known problems

#### Windows

- The temperature widget seems to require elevated access in some cases to get data.
- The battery widget seems to have issues with dual battery systems, like some Thinkpads.
- If you run on WSL/WSL2, you may have issues with getting memory data.
- WSL and WSL2 (as far as I know) cannot correctly report temperature sensors.
- WSL2 will not match Windows' own Task Manager in terms of data.

#### macOS

- The process widget may require elevated access (ex: `sudo btm`) to gather all data in some cases. _Please note that you should be certain that you trust any software you grant root privileges._

## Unofficial support

Systems and architectures that aren't officially supported may still work, but there are no guarantees on how much will work. For example, it might only compile, or it might run with bugs/broken features.
Furthermore, while it will depend on the problem at the end of the day, _issues on unsupported platforms are likely to go unfixed_.

Unofficially supported platforms known to compile/work:

- Linux on ARMv7 and ARMv6 (tested to compile in [CI](https://github.com/ClementTsang/bottom/blob/master/.github/workflows/ci.yml))
- macOS on AArch64 (tested to compile in [CI](https://github.com/ClementTsang/bottom/blob/master/.github/workflows/ci.yml))
- Linux on PowerPC 64 LE (tested to compile in [CI](https://github.com/ClementTsang/bottom/blob/master/.github/workflows/ci.yml))
- Linux on an RISC-V (tested to compile in [CI](https://github.com/ClementTsang/bottom/blob/master/.github/workflows/ci.yml), tested to run on an [Allwinner D1 Nezha](https://github.com/ClementTsang/bottom/issues/564))

### Known problems

- M1-based macOS devices may have issues with temperature sensors not returning anything.
