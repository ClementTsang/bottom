# Documentation

## When should documentation changes be done?

- Whenever a new feature is added, a bug is fixed, or a breaking change is made, it should be documented where appropriate (ex: `README.md`, changelog, etc.)
- New methods of installation are always appreciated and should be documented

## What pages need documentation?

There are a few areas where documentation changes are often needed:

- The [extended documentation](https://clementtsang.github.io/bottom/nightly/) (AKA here)
- The [`README.md`](https://github.com/ClementTsang/bottom/blob/master/README.md)
- The help menu inside of the application (located [here](https://github.com/ClementTsang/bottom/blob/master/src/constants.rs))
- The [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/master/CHANGELOG.md)

## How should I add documentation?

1. Fork the repository first and make changes there.

2. Where you're adding documentation will probably affect what you need to do:

   - For changes to [`README.md`](https://github.com/ClementTsang/bottom/blob/master/README.md) and [`CHANGELOG.md`](https://github.com/ClementTsang/bottom/blob/master/CHANGELOG.md), just follow the formatting provided and use any editor.

   - For changes to the help menu, try to refer to the existing code within `src/constants.rs` on how the help menu is generated.

   - For changes to the extended documentation, you'll want [MkDocs](https://www.mkdocs.org/), [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/), and `mdx_truly_sane_lists` installed to provide live reloading and preview for your changes. You can do so through `pip` or your system's package managers. While you don't _need_ these, it'll probably help in making and validating changes.

     You may also want [Mike](https://github.com/jimporter/mike), but it isn't really needed.

3. Once you have your documentation changes done, submit it as a pull request. For more information regarding that, refer to [Issues and Pull Requests](http://127.0.0.1:8000/contribution/issues-and-pull-requests/).
