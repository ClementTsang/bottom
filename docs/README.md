# Extended Documentation

This is where the extended documentation resides, hosted on GitHub Pages. We use [MkDocs](https://www.mkdocs.org/),
[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/), and [mike](https://github.com/jimporter/mike).

Documentation is currently built using Python 3.11, though it should work fine with older versions.

## Running locally

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

Deploying is done via [mike](https://github.com/jimporter/mike).

### Nightly

```bash
cd docs
mike deploy nightly --push
```

### Stable

```bash
cd docs

# Rename the previous stable version
mike retitle --push stable $OLD_STABLE_VERSION

# Set the newest version as the most recent stable version
mike deploy --push --update-aliases $RELEASE_VERSION stable

# Append a "(stable)" string to the end.
mike retitle --push $RELEASE_VERSION "$RELEASE_VERSION (stable)"
```
