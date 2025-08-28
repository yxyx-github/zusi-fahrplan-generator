use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::{InvalidBasePath, ZusiPath};

pub mod fahrplan_config;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields, rename = "ZusiEnvironment")]
pub struct ZusiEnvironmentConfig<T> {
    /// Path to own data directory root
    #[serde(rename = "@basePath")]
    pub base_path: PathBuf,

    #[serde(rename = "$value")]
    pub value: T,
}

impl<T> ZusiEnvironmentConfig<T> {
    pub fn into_zusi_environment(self, config_path: PathBuf) -> (ZusiEnvironment, T) {
        ZusiEnvironment::from_zusi_environment_config(self, config_path)
    }
}

pub struct ZusiEnvironment {
    /// Path to own data directory root
    pub base_path: PathBuf,

    /// Path to configuration file used as input
    pub config_path: PathBuf,
}

impl ZusiEnvironment {
    pub fn from_zusi_environment_config<T>(config: ZusiEnvironmentConfig<T>, config_path: PathBuf) -> (ZusiEnvironment, T) {
        (
            ZusiEnvironment {
                base_path: config.base_path,
                config_path,
            },
            config.value,
        )
    }
    
    pub fn path_to_prejoined_zusi_path<P: AsRef<Path> + Into<PathBuf>>(&self, path: P) -> Result<PrejoinedZusiPath, InvalidBasePath> {
        Ok(
            if path.as_ref().is_absolute() {
                PrejoinedZusiPath::new(&self.base_path, ZusiPath::new(path))
            } else {
                let root_path = self.config_path.join(&path);
                let zusi_path = ZusiPath::new_using_base(root_path, &self.base_path)?;
                PrejoinedZusiPath::new(&self.base_path, zusi_path)
            }
        )
    }
}

/*impl<T> From<ZusiEnvironmentConfig<T>> for (ZusiEnvironment, T) {
    fn from(value: ZusiEnvironmentConfig<T>) -> Self {
        (
            ZusiEnvironment {
                base_path: value.base_path,
            },
            value.value,
        )
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::cleanup_xml;
    use quick_xml::{de, se};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct ExampleValue {
        #[serde(rename = "$value")]
        pub value: String,
    }

    const EXPECTED_SERIALIZED: &'static str = r#"
        <ZusiEnvironment basePath="path/to/base">
            <ExampleValue>A</ExampleValue>
            <ExampleValue>B</ExampleValue>
            <ExampleValue>C</ExampleValue>
        </ZusiEnvironment>
    "#;

    fn expected_deserialized() -> ZusiEnvironmentConfig<Vec<ExampleValue>> {
        ZusiEnvironmentConfig {
            base_path: "path/to/base".into(),
            value: vec![
                ExampleValue { value: "A".into() },
                ExampleValue { value: "B".into() },
                ExampleValue { value: "C".into() },
            ],
        }
    }

    #[test]
    fn test_serialize() {
        let serialized = se::to_string(&expected_deserialized()).unwrap();
        assert_eq!(serialized, cleanup_xml(EXPECTED_SERIALIZED.into()));
    }

    #[test]
    fn test_deserialize() {
        let deserialized: ZusiEnvironmentConfig<Vec<ExampleValue>> = de::from_str(EXPECTED_SERIALIZED).unwrap();
        assert_eq!(deserialized, expected_deserialized());
    }
}