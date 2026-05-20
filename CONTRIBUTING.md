# Contribution

Contribution in any way is appreciated, whether it is reporting problems, fixing bugs, implementing features, improving the documentation, etc.

## Opening an issue

### Bug reports

When filing a bug report, fill out the [bug report template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=bug&template=bug_report.yml). Be sure to give all the necessary details! It is _incredibly_ difficult for a maintainer to fix a bug when it cannot be reproduced,
so that makes it much easier to reproduce the problem!

### Feature requests

Please fill out the [feature request template](https://github.com/ClementTsang/bottom/issues/new?assignees=&labels=feature&template=feature_request.yml). Remember to give details about what the feature is along with why you think this suggestion will be useful.

## Pull requests

The expected workflow for a pull request is:

1. Fork the project.
2. Make your changes.
3. Make any documentation changes if necessary - if you add a new feature, it'll probably need documentation changes. See [here](https://bottom.pages.dev/nightly/contribution/documentation/) for tips on documentation.
4. Commit and create a pull request to merge into the `main` branch. **Please fill out the pull request template and checklist**.
5. Ask a maintainer to review your pull request.
   - Check if the CI workflow passes. These consist of clippy lints, rustfmt checks, and basic tests. If you are a
     first-time contributor, you may need to wait for a maintainer to let CI run.
   - If changes are suggested or any comments are made, they should probably be addressed.
6. Once it looks good, it'll be merged! Note that generally, PRs are squashed to maintain repo cleanliness, though
   feel free to ask otherwise if that isn't preferable.

Some other things related to making changes or pull requests:

- Please review your code _yourself_ before you create the PR and ask for a review.
  - I'm fine if you use AI to help with your change, but I expect you to still review the changes yourself.
- If you are adding or changing any dependency, I expect a justification on why this is necessary.
- Please ensure that any changes made are compatible with the license of bottom.

For more details, see [here](https://bottom.pages.dev/nightly/contribution/issues-and-pull-requests/).

### Documentation

For contributing to documentation, see [here](https://bottom.pages.dev/nightly/contribution/documentation/).

### Packaging

If you want to become a package maintainer, see [here](https://bottom.pages.dev/nightly/contribution/packaging-and-distribution/)
for details on how to build bottom, how to generate/obtain completion files and manpages, and how to add installation instructions for the package to the README.
