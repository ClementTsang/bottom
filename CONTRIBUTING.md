# Contribution

If you want to contribute to this project - first of all, thank you! I'm glad to see interest in it!

Here are some notes about how to contribute to bottom (structure is based on the official
[Rust contribution guidelines](https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md)):

- [Feature reports](#feature-reports)
- [Bug reports](#bug-reports)
- [Other types of reports](#other-types-of-reports)
- [Pull requests](#pull-requests)

## Feature reports

Feature suggestions can be submitted using the "feature" tag. Prior to submission, please look to see if this has already been suggested or solved; if it has and is not resolved, it would be better to comment on the relevant report.

Within your feature report, try to answer the given prompts - in particular, state the specific feature you want and if possible, please state why you want this added to the program.

## Bug reports

Bug reports can be submitted using the "bug" tag. Prior to submission, please look to see if this has already been reported or solved; if it has and is not resolved, it would be better to comment on the relevant report.

Within your bug report, try to answer the given prompts. Be as specific as possible - describe your bug to the best of your ability, how to replicate it, and provide information like screenshots, OS and terminal. It can be very useful to help whoever is dealing with the issue!

## Other types of reports

For reports/suggestions that don't fit the definition of a feature or bug, try to use the other tags:

- `documentation`: If you note a typo, or want to suggest something to do with the documentation of bottom, use this.
- `question`: If you have a question, and not really a suggestion or request, then use this tag.
- `ci`, `investigative`, `refactoring`: Generally, these are for internal use to track issues in order to manage GitHub Projects, and won't be the appropriate topic for a report.
- `other`: If your suggestion or issue doesn't fit those categories, then use this.

## Pull requests

If you want to help contribute by submitting a PR, by all means, I'm open! In regards to the development process:

- I develop primarily using _stable_ Rust. That is, whatever is the most up-to-date stable version you can get via running
  `rustup update stable`.

- There are some tests, they're mostly for sanity checks. Please run `cargo test` to ensure you didn't break anything important, unless the change will break the test (in which case please amend the tests).

  - Note that `cargo test` will fail on anything lower than 1.43.0 due to it using a then-introduced env variable.

- I use both [clippy](https://github.com/rust-lang/rust-clippy) and [rustfmt](https://github.com/rust-lang/rustfmt) in development (with some settings, see [clippy.toml](./clippy.toml) and [rustfmt.toml](rustfmt.toml)). Note clippy must pass to for PRs to be accepted.

  - You can check clippy using `cargo clippy`.

  - I use [cargo-husky](https://github.com/rhysd/cargo-husky) to automatically run a clippy check on push. You can disable this in the `Cargo.toml` file if you find this annoying.

- You may notice that I have fern and log as dependencies; this is mostly for easy debugging via the `debug!()` macro. It writes to the
  `debug.log` file that will automatically be created if you run in debug mode (so `cargo run`).

And in regards to the pull request process:

- Create a personal fork of the process and PR that, as per the [fork and pull method](https://help.github.com/en/github/collaborating-with-issues-and-pull-requests/about-collaborative-development-models).

- Merge against the `master` branch.

- If you encounter a merge conflict, I expect you to resolve this via rebasing to `master`.

- _Ensure your change builds, runs, and works_. Furthermore, state how you checked this, including what platforms you tested on.

- If your change will result in needing to update documentation, please do so. In particular:

  - Does the README need to be updated to accommodate your change?

  - Does the help action, `?`, need to be updated to accommodate your change?

- Please ensure that CI passes. If it fails, check to see why it fails! Chances are it's clippy.

- If all looks good, then request someone with write access (so basically me, [@ClementTsang](https://github.com/ClementTsang)) to give your code a review. If it's fine, then I'll merge!

- Please use the [pull request template](https://github.com/ClementTsang/bottom/blob/master/.github/pull_request_template.md) to help you.
