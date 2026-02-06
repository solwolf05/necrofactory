use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub dependencies: HashMap<String, String>,
    pub optional_dependencies: HashMap<String, String>,
}

impl<'de> Deserialize<'de> for Metadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMetadata {
            pub id: String,
            pub name: String,
            pub version: String,
            pub author: String,
            pub dependencies: Option<HashMap<String, String>>,
            pub optional_dependencies: Option<HashMap<String, String>>,
        }

        let raw = RawMetadata::deserialize(deserializer)?;
        Ok(Metadata {
            id: raw.id,
            name: raw.name,
            version: raw.version,
            author: raw.author,
            dependencies: raw.dependencies.unwrap_or_default(),
            optional_dependencies: raw.optional_dependencies.unwrap_or_default(),
        })
    }
}
