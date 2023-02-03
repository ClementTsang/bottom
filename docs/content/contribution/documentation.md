# Documentation

## When should documentation changes be done?

- Whenever a new feature is added, a bug is fixed, or a breaking change is made, it should be documented where
  appropriate (ex: `README.md`, changelog, etc.)
- New methods of installation are always appreciated and should be documented

## What pages need documentation?

There are a few areas where documentation changes are often needed:

- The [`README.md`](https://github.com/ClementTsang/bottom/blob/master/README.md)
- The help menu inside of the application (located [here](https://github.com/ClementTsang/bottom/blob/master/src/constants.rs))
- The [extended documentation](https://clementtsang.github.io/bottom/nightly/) (here)
- The [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/master/CHANGELOG.md)

## How should I add/update documentation?

1. Fork the repository to make changes in.

2. Where you're adding documentation will probably affect what you need to do:

   <h3><code>README.md</code> or <code>CHANGELOG.md</code></h3>

   For changes to [`README.md`](https://github.com/ClementTsang/bottom/blob/master/README.md) and [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/master/CHANGELOG.md), just follow the formatting provided and use any editor.

   Generally, changes to [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/master/CHANGELOG.md) will be handled
   by a maintainer, and changes should follow the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format, as
   well as link to the relevant PR or issue.

   <h3>Help menu</h3>

   For changes to the help menu, try to refer to the existing code within `src/constants.rs` on how the help menu is generated.

   <h3>Extended documentation</h3>

   For changes to the extended documentation, you'll probably want Python 3.11 (older versions should be fine though),
   [MkDocs](https://www.mkdocs.org/), [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/),
   `mdx_truly_sane_lists`, and optionally [Mike](https://github.com/jimporter/mike) installed to provide live reloading
   and preview for your changes. They aren't needed but it'll help with validating your changes.

   You can do so through `pip` or your system's package managers. If you use `pip`, you can use venv to cleanly install
   the documentation dependencies:

   ```bash
   # Change directories to the documentation.
   cd docs/

   # Create and activate venv.
   python -m venv venv
   source venv/bin/activate

   # Install requirements
   pip install -r requirements.txt

   # Run mkdocs
   venv/bin/mkdocs serve
   ```

   This will serve a local version of the docs that you can open on your browser. It will update as you make changes.

3. Once you have your documentation changes done, submit it as a pull request. For more information regarding that,
   refer to [Issues, Pull Requests, and Discussions](../issues-and-pull-requests/).
