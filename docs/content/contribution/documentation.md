# Documentation

## When should documentation changes be done?

- Whenever a new feature is added, a bug is fixed, or a breaking change is made, it should be documented where
  appropriate (ex: `README.md`, changelog, etc.)
- New methods of installation are always appreciated and should be documented

## What pages need documentation?

There are a few areas where documentation changes are often needed:

- The [`README.md`](https://github.com/ClementTsang/bottom/blob/main/README.md)
- The help menu inside of the application (located [here](https://github.com/ClementTsang/bottom/blob/main/src/constants.rs))
- The [extended documentation](https://clementtsang.github.io/bottom/nightly/) (here)
- The [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/main/CHANGELOG.md)

## How should I add/update documentation?

1. Fork the repository to make changes in.

2. Where you're adding documentation will probably affect what you need to do:

   <h3><code>README.md</code> or <code>CHANGELOG.md</code></h3>

   For changes to [`README.md`](https://github.com/ClementTsang/bottom/blob/main/README.md) and [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/main/CHANGELOG.md), just follow the formatting provided and use any editor.

   Generally, changes to [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/main/CHANGELOG.md) will be handled
   by a maintainer, and the contents of the file should follow the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
   format, as well as link to the relevant PR or issues.

   <h3>Help menu</h3>

   For changes to the help menu, try to refer to the existing code within [`src/constants.rs`](https://github.com/ClementTsang/bottom/blob/main/src/constants.rs) on how the help menu is generated.

   <h3>Extended documentation</h3>

   For changes to the extended documentation, you'll probably want at least Python 3.11 (older and newer versions
   should be fine), [MkDocs](https://www.mkdocs.org/), [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/),
   `mdx_truly_sane_lists`, and optionally [Mike](https://github.com/jimporter/mike) installed. These can help with
   validating your changes locally.

   You can do so through `pip` or your system's package managers. If you use `pip`, you can use venv to cleanly install
   the documentation dependencies:

   ```bash
   # Change directories to the documentation.
   cd docs/

    # Create venv, install the dependencies, and serve the page.
   ./serve.sh
   ```

   This will serve a local version of the docs that you can open on your browser. It will update as you make changes.

3. Once you have your documentation changes done, submit it as a pull request. For more information regarding that,
   refer to [Issues, Pull Requests, and Discussions](issues-and-pull-requests.md).
