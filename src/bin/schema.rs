#![cfg(feature = "generate_schema")]

use bottom::{options::config, widgets};
use clap::Parser;
use itertools::Itertools;
use strum::VariantArray;

#[derive(Parser)]
struct SchemaOptions {
    /// The version of the schema.
    version: Option<String>,
}

fn generate_schema(schema_options: SchemaOptions) -> anyhow::Result<()> {
    let mut schema = schemars::schema_for!(config::Config);
    {
        // TODO: Maybe make this case insensitive? See https://stackoverflow.com/a/68639341

        let proc_columns = schema.definitions.get_mut("ProcColumn").unwrap();
        match proc_columns {
            schemars::schema::Schema::Object(proc_columns) => {
                let enums = proc_columns.enum_values.as_mut().unwrap();
                *enums = widgets::ProcColumn::VARIANTS
                    .iter()
                    .flat_map(|var| var.get_schema_names())
                    .sorted()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .dedup()
                    .collect();
            }
            _ => anyhow::bail!("missing proc columns definition"),
        }

        let disk_columns = schema.definitions.get_mut("DiskColumn").unwrap();
        match disk_columns {
            schemars::schema::Schema::Object(disk_columns) => {
                let enums = disk_columns.enum_values.as_mut().unwrap();
                *enums = widgets::DiskColumn::VARIANTS
                    .iter()
                    .flat_map(|var| var.get_schema_names())
                    .sorted()
                    .map(|v| serde_json::Value::String(v.to_string()))
                    .dedup()
                    .collect();
            }
            _ => anyhow::bail!("missing disk columns definition"),
        }
    }

    let metadata = schema.schema.metadata.as_mut().unwrap();
    let version = schema_options.version.unwrap_or("nightly".to_string());
    metadata.id = Some(format!(
        "https://github.com/ClementTsang/bottom/blob/main/schema/{version}/bottom.json"
    ));
    metadata.description = Some(format!(
        "https://clementtsang.github.io/bottom/{}/configuration/config-file",
        if version == "nightly" {
            "nightly"
        } else {
            "stable"
        }
    ));
    metadata.title = Some(format!("Schema for bottom's config file ({version})",));
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let schema_options = SchemaOptions::parse();
    generate_schema(schema_options)?;

    Ok(())
}
