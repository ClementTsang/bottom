# Developer Tips

!!! Warning

    This section is intended for people who wish to work on/build/distribute bottom, not general users.
    Also note that this doc will change over time and is somewhat always a work-in-progress.

This section of the docs provides some basic guidance, tips, and tricks for working on bottom.

## Rust setup

Follow the standard [Rust installation guide](https://rust-lang.org/tools/install/) to set Rust up. Development
is typically done on the latest stable installation.

## Profiling

I've personally used a few tools for profiling:

- [**Samply**](https://github.com/mstange/samply): I like this one a lot for CPU profiling, since it works with the
  Firefox profiler for a nice interface.
- [**DHAT**](https://valgrind.org/docs/manual/dh-manual.html): This is useful for heap analysis. The dhat allocator is
  actually already in `src/lib.rs`, albeit commented out. To use it, pull in the [dhat crate](https://crates.io/crates/dhat)
  and uncomment the following lines:

  ```rust
  static ALLOC: dhat::Alloc = dhat::Alloc

  //...
  
  let _profiler = dhat::Profiler::new_heap();
  ```

## Build cache

A local build cache can really speed things up while developing. While Cargo's built-in cache can suffice if you're
just doing simple changes, if you're working on multiple changes over a few worktrees, there can be a lot of wasted
space and time. At the moment I really like [Kache](https://github.com/kunobi-ninja/kache).
[sccache](https://github.com/mozilla/sccache) is pretty good too. Follow their guides on setting things up if you
think you would find this useful - note these are also pretty useful for general Rust dev too!

### Kache

A note on Kache - if you use this, disable the test runner env vars, as I've found they don't work on at least Linux.
