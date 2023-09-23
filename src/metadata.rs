use serde_derive::Deserialize;
use std::path::PathBuf;

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
    pub manifest_path: PathBuf,
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
