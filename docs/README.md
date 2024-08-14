# Extended Documentation

This is where the extended documentation resides, hosted on GitHub Pages. We use [MkDocs](https://www.mkdocs.org/),
[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/), and [mike](https://github.com/jimporter/mike).

Documentation is currently built using Python 3.11, though it should work fine with older versions.

## Running locally

One way is to just run `serve.sh`. Alternatively, the manual steps are, assuming your current working directory
is the bottom repo:

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

## Deploying

Deploying is done via [mike](https://github.com/jimporter/mike) in order to get versioning. Typically,
this is done through CI, but can be done manually if needed.

### Nightly docs

```bash
cd docs
mike deploy nightly --push
```

### Stable docs

```bash
cd docs

# Rename the previous stable version
mike retitle --push stable $OLD_STABLE_VERSION

# Set the newest version as the most recent stable version
mike deploy --push --update-aliases $RELEASE_VERSION stable

# Append a "(stable)" string to the end.
mike retitle --push $RELEASE_VERSION "$RELEASE_VERSION (stable)"
```
