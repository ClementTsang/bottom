# Deploy Process

!!! Warning

    This section is currently WIP.

!!! Warning

    This section is intended for people who wish to work on/build/distribute bottom, not general users.

## Overview

bottom currently has two main deploy processes to worry about:

- [Nightly](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/nightly.yml): a daily (00:00 UTC) GitHub action to build binary/installer files, and upload them to the [nightly release](https://github.com/ClementTsang/bottom/releases/tag/nightly). It can also be triggered manually as either a proper nightly release or a mock release.
- [Stable](https://github.com/ClementTsang/bottom/blob/main/.github/workflows/deployment.yml): a stable deployment, triggered manually or upon creation of a valid tag. This is a GitHub action that builds binary/installer files and uploads them to a new GitHub release.

  Furthermore, this workflow does not handle the following deployments, which must be manually handled:

  - [Chocolatey](https://community.chocolatey.org/packages/bottom)
  - [crates.io](https://crates.io/crates/bottom)

## Nightly

This is, for the most part, automatic, though it can also be used as a way of testing build workflow changes and seeing if binaries can be successfully built at all against all the targets we want to build for.

If one does not want to actually update the nightly release, and just want to test the general builds and workflow, one can run the workflow manually on a branch of choice with "mock" set as the parameter. Changing it to anything else will trigger a non-mock run.

## Stable

This can be manually triggered, though the general use-case is setting a tag of the form `x.y.z` (after checking everything is good, of course). For example:

```bash
git tag 0.6.9 && git push origin 0.6.9
```

This will automatically trigger the deployment workflow, and create a draft release with the files uploaded. One still needs to fill in the details and release it.

Furthermore, there are some deployments that are handled by maintainers of bottom that this workflow does not automatically finish. These must be manually handled.

### Chocolatey

Upon releasing on GitHub, [choco-bottom](https://github.com/ClementTsang/choco-bottom) will automatically be updated with a new PR with the correct deployment files for Chocolatey. Check the PR, merge it if it is correct, then pull locally and deploy following the instructions in the [README](https://github.com/ClementTsang/choco-bottom/blob/master/README.md). Make sure to test installation and running at least once before deploying!

If done correctly, there should be a new build on Chocolatey, which will take some time to validate.

### crates.io

Validate everything builds properly and works (you should have done this before releasing though). If good, then deploying on crates.io is as simple as:

```bash
cargo publish
```
