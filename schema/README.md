# Config JSON Schema

## Generation

These are automatically generated from code using [`schemars`](https://github.com/GREsau/schemars). They're locked
behind a feature flag to avoid building unnecessary code for release builds, and you can generate them like so:

```bash
# Will print out to stdout
cargo run --features="generate_schema" -- --generate_schema

# e.g. for nightly
cargo run --features="generate_schema" -- --generate_schema > schema/nightly/bottom.json

# e.g. for a specific version
cargo run --features="generate_schema" -- --generate_schema 0.12.0 > schema/v0.12.0/bottom.json
```

Alternatively, run the `scripts/schema/generate.sh` script (for stable releases) or `scripts/schema/nightly.sh`
(for nightly), which does all of this for you.

## Publication

To publish these schemas:

### Stable

1. Run `scripts/schema/generate.sh <YOUR_VERSION>`.
2. Make a PR and merge it.
3. Then, make a PR to [schemastore](https://github.com/SchemaStore/schemastore) to update the catalog.
   [Here's an example of a PR](https://github.com/SchemaStore/schemastore/pull/5242).

### Nightly

1. Run `scripts/schema/nightly.sh`.
2. Make a PR and merge it.
