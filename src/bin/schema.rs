#![cfg(feature = "generate_schema")]
#![expect(
    clippy::unwrap_used,
    reason = "this is just used to generate jsonschema files"
)]

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

macro_rules! generate_column_schemas {
    ($struct_name:literal, $variants:expr, $schema:expr) => {
        match $schema
            .as_object_mut()
            .unwrap()
            .get_mut("$defs")
            .unwrap()
            .get_mut($struct_name)
            .unwrap()
        {
            Value::Object(original) => {
                let enums = original.get_mut("enum").unwrap();
                *enums = $variants
                    .iter()
                    .flat_map(|variant| variant.get_schema_names())
                    .flat_map(|variant| [variant.to_string(), variant.to_lowercase()])
                    .sorted() // Remember that dedup only works if it's sorted...
                    .dedup()
                    .map(|variant| serde_json::Value::String(variant)) // Have to do it after as it doesn't implement partialeq/eq
                    .collect();

                Ok(())
            }
            _ => Err(anyhow::anyhow!("missing proc columns definition")),
        }
    };
}

fn generate_schema(schema_options: SchemaOptions) -> anyhow::Result<()> {
    let mut schema = schemars::schema_for!(config::Config);
    {
        // TODO: Maybe make this case insensitive? See https://stackoverflow.com/a/68639341
        generate_column_schemas!("ProcColumn", widgets::ProcColumn::VARIANTS, schema)?;
        generate_column_schemas!(
            "DiskWidgetColumn",
            widgets::DiskWidgetColumn::VARIANTS,
            schema
        )?;
        generate_column_schemas!(
            "TempWidgetColumn",
            widgets::TempWidgetColumn::VARIANTS,
            schema
        )?;
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
                version.as_str()
            }
        )
        .into(),
    );

    let description_version = if version == "nightly" {
        "nightly".to_string()
    } else {
        format!("v{version}")
    };
    schema.insert(
        "title".into(),
        format!("Schema for bottom's config file ({description_version})").into(),
    );

    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let schema_options = SchemaOptions::parse();
    generate_schema(schema_options)?;

    Ok(())
}
