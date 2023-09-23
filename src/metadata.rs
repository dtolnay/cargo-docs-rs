use serde::de::{Deserialize, Deserializer};
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub packages: Vec<Package>,
    pub workspace_members: Vec<PackageId>,
    pub resolve: Resolve,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub id: PackageId,
    #[serde(deserialize_with = "deserialize_docs_rs")]
    pub metadata: DocumentationOptions,
}

#[derive(Deserialize, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct PackageId {
    pub repr: String,
}

#[derive(Deserialize, Debug)]
pub struct Resolve {
    pub root: Option<PackageId>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DocumentationOptions {
    #[serde(default)]
    pub features: Vec<String>,

    #[serde(default)]
    pub all_features: bool,

    #[serde(default)]
    pub no_default_features: bool,

    pub default_target: Option<String>,
    pub targets: Option<Vec<String>>,

    #[serde(default)]
    pub rustc_args: Vec<String>,

    #[serde(default)]
    pub rustdoc_args: Vec<String>,

    #[serde(default)]
    pub cargo_args: Vec<String>,
}

fn deserialize_docs_rs<'de, D>(deserializer: D) -> Result<DocumentationOptions, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Outer {
        pub docs: Option<Inner>,
    }

    #[derive(Deserialize)]
    struct Inner {
        pub rs: Option<DocumentationOptions>,
    }

    let outer: Option<Outer> = Deserialize::deserialize(deserializer)?;
    if let Some(outer) = outer {
        if let Some(inner) = outer.docs {
            if let Some(options) = inner.rs {
                return Ok(options);
            }
        }
    }

    Ok(DocumentationOptions::default())
}
