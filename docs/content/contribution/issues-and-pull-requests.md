# Issues and Pull Requests

Contribution in any way is appreciated, whether it is reporting problems, fixing bugs, implementing features, improving the documentation, etc.

## Opening an issue

### Bug reports

When filing a bug report, please use the [bug report template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=bug&template=bug_report.md&title=) and fill in as much as you can. It is _incredibly_ difficult for a maintainer to fix a bug when it cannot be reproduced, and giving as much detail as possible generally helps to make it easier to reproduce the problem!

### Feature requests

Please use the [feature request template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=feature&template=feature_request.md&title=) and fill it out. Remember to give details about what the feature is along with why you think this suggestion will be useful.

Also please check whether or not an existing issue has covered your specific feature request!

## Pull requests

The expected workflow for a pull request is:

1. Fork the project.
2. Make your changes.
3. Make any documentation changes if necessary - if you add a new feature, it'll probably need documentation changes. See [here](./documentation.md) for tips on documentation.
4. Commit and create a pull request to merge into the `master` branch. **Please follow the pull request template**.
5. Wait for the tests to pass. These consist of clippy lints, rustfmt checks, and basic tests. **If you are a first time contributor, you may need to skip this step for now, as GitHub Actions requires approval to run.**
6. Ask a maintainer to review your pull request. If changes are suggested or any comments are made, they should probably be addressed. Once it looks good, it'll be merged!
