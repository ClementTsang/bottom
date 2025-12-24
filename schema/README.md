# Config JSON Schema

## Generation

These are automatically generated from code using [`schemars`](https://github.com/GREsau/schemars). They're locked
behind a feature flag to avoid building unnecessary code for release builds, and you can generate them like so:

```bash
# Will print out to stdout
cargo run --features="generate_schema" -- --generate_schema

# e.g. for nightly
cargo run --features="generate_schema" -- --generate_schema > schema/nightly/bottom.json
```

Alternatively, run the `scripts/schema/generate.sh` script (for stable releases) or `scripts/schema/nightly.sh`
(for nightly).

## Publication

To publish these schemas, cut a new version by copying `nightly` to a new folder with a version number matching bottom's
(e.g. v0.10 if bottom is on v0.10.x bottom). Then, make a PR to [schemastore](https://github.com/SchemaStore/schemastore)
updating the catalog.

For more info, see the schemastore repo. [Here's an example of a PR](https://github.com/SchemaStore/schemastore/pull/3571).
