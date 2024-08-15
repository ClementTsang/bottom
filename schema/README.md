# Config JSON Schema

## Generation

These are automatically generated from code using [`schemars`](https://github.com/GREsau/schemars). They're locked
behind a feature flag to avoid building unnecessary code for release builds, and you can generate them like so:

```bash
cargo run --features="generate_schema" -- --generate_schema > schema/nightly/bottom.json
```

Alternatively, run the script in `scripts/schema/generate.sh`, which does this for you.

## Publication

To publish these schemas, cut a new version by copying `nightly` to a new folder with a version number matching bottom's
(e.g. v0.10 if bottom is on v0.10.x bottom). Then, make a PR to [schemastore](https://github.com/SchemaStore/schemastore)
updating the catalog.

For more info, see the schemastore repo. An example PR can be found [here](https://github.com/SchemaStore/schemastore/pull/3571).
