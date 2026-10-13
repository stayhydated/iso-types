//! Load the generated selection index once per macro process.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;
use syn::LitStr;

pub(super) struct Entry {
    pub code: String,
    pub variant: String,
}

pub(super) struct Catalog {
    pub type_name: String,
    pub feature: String,
    pub entries: Vec<Entry>,
}

fn string(value: &Value, key: &str) -> Result<String, String> {
    value[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("expected catalog string field `{key}`"))
}

fn load() -> Result<BTreeMap<String, Catalog>, String> {
    let value: Value =
        serde_json::from_str(include_str!("data.json")).map_err(|error| error.to_string())?;
    let object = value.as_object().ok_or("expected catalog map")?;
    object
        .iter()
        .map(|(standard, value)| {
            let entries = value["entries"]
                .as_array()
                .ok_or("expected catalog entries")?
                .iter()
                .map(|entry| {
                    Ok(Entry {
                        code: string(entry, "code")?,
                        variant: string(entry, "variant")?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            if entries.is_empty() || entries.windows(2).any(|pair| pair[0].code >= pair[1].code) {
                return Err(format!("{standard} entries must have unique sorted codes"));
            }
            Ok((
                standard.clone(),
                Catalog {
                    type_name: string(value, "type")?,
                    feature: string(value, "feature")?,
                    entries,
                },
            ))
        })
        .collect()
}

pub(super) fn catalog(standard: &LitStr) -> syn::Result<&'static Catalog> {
    static CATALOGS: OnceLock<Result<BTreeMap<String, Catalog>, String>> = OnceLock::new();
    let catalogs = CATALOGS.get_or_init(load).as_ref().map_err(|error| {
        syn::Error::new_spanned(standard, format!("invalid generated catalog: {error}"))
    })?;
    catalogs.get(&standard.value()).ok_or_else(|| {
        let supported = catalogs
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        syn::Error::new_spanned(
            standard,
            format!("unsupported catalog; choose one of: {supported}"),
        )
    })
}
