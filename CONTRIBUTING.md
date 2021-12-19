# Contribution

Contribution in any way is appreciated, whether it is reporting problems, fixing bugs, implementing features, improving the documentation, etc.

## Opening an issue

### Bug reports

When filing a bug report, use the [bug report template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=bug&template=bug_report.md&title=)
and fill in as much as you can. It is _incredibly_ difficult for a maintainer to fix a bug when it cannot be reproduced,
and giving as much detail as possible generally helps to make it easier to reproduce the problem!

### Feature requests

Please use the [feature request template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=feature&template=feature_request.md&title=) and fill it out. Remember to give details about what the feature is along with why you think this suggestion will be useful.

## Pull requests

If you want to directly contribute documentation changes or code, follow this! The expected workflow for a pull request is:

1. Fork the project.
2. Make your changes locally.
3. Commit and create a pull request to merge into the `master` branch. **Please follow the pull request template**.
4. Wait for the tests to pass. These consist of clippy lints, rustfmt checks, and basic tests. **If you are a first time contributor, skip to the next step for now, as GitHub Actions requires approval to run.**
5. Ask a maintainer to review your pull request. If changes are suggested or any comments are made, they should probably be addressed. Once it looks good, it'll be merged!

For more details, see [here](https://clementtsang.github.io/bottom/nightly/contribution/issues-and-pull-requests/).

### Documentation

For contributing to documentation, see [here](https://clementtsang.github.io/bottom/nightly/contribution/documentation/).

### Packaging

If you want to become a package maintainer, look [here](https://clementtsang.github.io/bottom/nightly/contribution/packaging-and-distribution/)
for instructions on how to build bottom and add installation instructions to the README.
