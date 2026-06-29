//! JSON Schema generation for herdr's `config.toml`.
//!
//! Everything in this module compiles only when the `schema` cargo feature is
//! enabled, so release builds do not pull in `schemars`. The `herdr config
//! schema` subcommand and the schema drift test gate on the same feature.

#[cfg(feature = "schema")]
use super::model::{ClipboardToastConfig, Config, HerdrToastConfig, ToastDelivery};

/// Schema-shape mirror of `[ui.toast]`'s custom deserializer.
///
/// `ToastConfig` has a custom `Deserialize` impl that accepts an optional
/// legacy `enabled` bool alongside `delivery`, and rejects `delay_seconds`
/// outside `0..=3600`. Routing `JsonSchema` for `ToastConfig` through this
/// helper (via `#[schemars(from = "ToastConfigSchema")]`) makes the generated
/// schema reflect the actual accepted TOML shape instead of the runtime field
/// shape.
#[cfg(feature = "schema")]
#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub(crate) struct ToastConfigSchema {
    pub delivery: Option<ToastDelivery>,
    /// Legacy boolean: `true` maps to `delivery = "herdr"`, `false` to "off".
    /// Ignored when `delivery` is set explicitly.
    pub enabled: Option<bool>,
    /// Seconds before a toast is dismissed. Must be between 0 and 3600.
    #[cfg_attr(feature = "schema", schemars(range(min = 0, max = 3600)))]
    pub delay_seconds: Option<u64>,
    pub herdr: HerdrToastConfig,
    pub clipboard: ClipboardToastConfig,
}

/// Generate the JSON Schema (Draft 2020-12) for `config.toml`.
///
/// The schema describes the *deserialize* contract, so serde `alias`,
/// `default`, and `skip_deserializing` attributes are honored.
#[cfg(feature = "schema")]
pub(crate) fn generate_schema() -> serde_json::Value {
    let schema = schemars::schema_for!(Config);
    let mut value =
        serde_json::to_value(&schema).expect("generated config schema must be serializable");
    if let Some(map) = value.as_object_mut() {
        map.insert(
            "$id".into(),
            serde_json::Value::String("https://herdr.dev/schemas/config.schema.json".into()),
        );
    }
    value
}

/// Pretty-printed schema text used by both the CLI subcommand and the drift
/// test, so they share one canonical formatting.
#[cfg(feature = "schema")]
pub(crate) fn generate_schema_pretty() -> String {
    let value = generate_schema();
    let mut pretty =
        serde_json::to_string_pretty(&value).expect("generated config schema must be serializable");
    pretty.push('\n');
    pretty
}

#[cfg(all(test, feature = "schema"))]
mod tests {
    use super::generate_schema_pretty;

    const COMMITTED_SCHEMA: &str = include_str!("../../schemas/config.json");

    #[test]
    fn schema_matches_committed_file() {
        let generated = generate_schema_pretty();

        if std::env::var_os("HERDR_REGEN_SCHEMA").is_some() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/config.json");
            std::fs::write(&path, &generated)
                .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
            eprintln!("regenerated {}", path.display());
            return;
        }

        assert_eq!(
            generated, COMMITTED_SCHEMA,
            "committed schemas/config.json differs from the schema generated \
             from the Rust config types. Regenerate with:\n  \
             HERDR_REGEN_SCHEMA=1 cargo test --features schema schema_matches_committed_file"
        );
    }
}
