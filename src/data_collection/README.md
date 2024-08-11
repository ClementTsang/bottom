# Data Collection

Data collection in bottom has two main components: **sources** and **collectors**.

**Sources** are either libraries or system APIs that actually extract the data.
These may map to multiple different operating systems. Examples are `sysinfo`,
or `libc` bindings, or Linux-specific code.

**Collectors** are _platform-specific_ (typically OS-specific), and can pull from
different sources to get all the data needed, with some glue code in between. As
such, sources should be written to be per-"job", and be divisible such that
collectors can import specific code as needed.

We can kinda visualize this with a quick-and-dirty diagram (note this is not accurate or up-to-date):

```mermaid
flowchart TB
    subgraph sources
        direction TB
        linux
        windows
        macos
        unix
        sysinfo
        freebsd
    end
    subgraph collectors
        direction TB
        Linux
        Windows
        macOS
        FreeBSD
    end
    linux -..-> Linux
    unix -..-> Linux
    sysinfo -..-> Linux
    windows -..-> Windows
    sysinfo -..-> Windows
    macos -..-> macOS
    unix -..-> macOS
    sysinfo -..-> macOS
    freebsd -..-> FreeBSD
    sysinfo -..-> FreeBSD
```
