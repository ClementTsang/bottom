# Official support

bottom _officially_ supports the following operating systems and corresponding architectures:

- macOS (`x86_64`, `aarch64`)
- Linux (`x86_64`, `i686`, `aarch64`)
- Windows (`x86_64`, `i686`)

These platforms are tested to work (with caveats, see below) and issues on these platforms will be fixed if possible.

Furthermore, binaries are expected to be built and tested using the most recent version of stable Rust - if you are manually building
bottom from the repo/source, then please try that as well.

## Known problems

### Linux

- If you're using Linux via WSL or WSL2:
  - You may have issues with getting memory data.
  - Temperature sensors may not be correctly reported.
  - WSL2 will not match Windows' own Task Manager in terms of data.

### Windows

- The temperature widget seems to require admin privileges in some cases to get data.
- The battery widget seems to have issues with dual battery systems, like some Thinkpads.
- If you are using WSL or WSL2:
  - You may have issues with getting memory data.
  - Temperature sensors may not be correctly reported.
  - WSL2 will not match Windows' own Task Manager in terms of data.

### macOS

- The process widget may require elevated access (ex: `sudo btm`) to gather all data in some cases. _Please note that you should be certain that you trust any software you grant root privileges._
