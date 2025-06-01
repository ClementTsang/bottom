#![cfg(feature = "generate_schema")]

use bottom::{options::config, widgets};
use clap::Parser;
use itertools::Itertools;
use serde_json::Value;
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

        match schema
            .as_object_mut()
            .unwrap()
            .get_mut("$defs")
            .unwrap()
            .get_mut("ProcColumn")
            .unwrap()
        {
            Value::Object(proc_columns) => {
                let enums = proc_columns.get_mut("enum").unwrap();
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

        match schema
            .as_object_mut()
            .unwrap()
            .get_mut("$defs")
            .unwrap()
            .get_mut("DiskColumn")
            .unwrap()
        {
            Value::Object(disk_columns) => {
                let enums = disk_columns.get_mut("enum").unwrap();
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

    let version = schema_options.version.unwrap_or("nightly".to_string());
    schema.insert(
        "$id".into(),
        format!("https://github.com/ClementTsang/bottom/blob/main/schema/{version}/bottom.json")
            .into(),
    );

    schema.insert(
        "description".into(),
        format!(
            "https://bottom.pages.dev/{}/configuration/config-file/",
            if version == "nightly" {
                "nightly"
            } else {
                "stable"
            }
        )
        .into(),
    );

    schema.insert(
        "title".into(),
        format!("Schema for bottom's config file ({version})").into(),
    );

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let schema_options = SchemaOptions::parse();
    generate_schema(schema_options)?;

    Ok(())
}
