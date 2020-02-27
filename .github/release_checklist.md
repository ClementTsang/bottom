# Release checklist

This checklist is mostly for me personally - just want to have a easy to refer to record for what I should do before I release.

## General

- Did travis pass (obviously)?

- Did you uptick the version?

- Is every feature tested?

- Are any new bugs introduced in the core features?

- Did you test:

  - Searching

    - Do the modifiers work?

    - Do all chars work?

  - Basic widget movement

  - Flags

  - Config files

    - Colouring

- Did you _really_ test all this?

- Did you test `cargo install` (I don't want to ever have to deal with that fiasco again, jeez)?

- Is documentation up to spec?

- If everything is good, create a release branch!

## Linux

### Arch

- Did you edit the PKGBUILD with the correct version + hash?

- Did you add a .SRCINFO?

- Did you test it?

- Did you commit it all?

### Debian

- Did you edit the new `.deb` file version in?

## macOS

- Did you update the `bottom.rb` file with the correct version + hash?

- Did you test it?

## Windows

- Did you edit the nupkg?

- Did you test if it works?
