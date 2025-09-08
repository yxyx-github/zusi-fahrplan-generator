use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RollingStockConfig {
    #[serde(rename = "@path")]
    pub path: PathBuf,
}