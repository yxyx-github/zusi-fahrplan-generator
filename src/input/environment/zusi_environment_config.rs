use std::fmt::Debug;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::core::lib::file_error::FileError;
use crate::input::environment::zusi_environment::ZusiEnvironment;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields, rename = "ZusiEnvironment")]
pub struct ZusiEnvironmentConfig<T> {
    /// Path to own data directory root
    #[serde(rename = "@dataDir")]
    pub data_dir: PathBuf,

    #[serde(rename = "$value")]
    pub value: T,
}

impl<T> ZusiEnvironmentConfig<T> {
    pub fn into_zusi_environment(self, config_path: PathBuf) -> Result<(ZusiEnvironment, T), FileError> {
        ZusiEnvironment::from_zusi_environment_config(self, config_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct ExampleValue {
        #[serde(rename = "$value")]
        pub value: String,
    }

    const EXPECTED_SERIALIZED: &'static str = r#"
        <ZusiEnvironment dataDir="path/to/data_dir">
            <ExampleValue>A</ExampleValue>
            <ExampleValue>B</ExampleValue>
            <ExampleValue>C</ExampleValue>
        </ZusiEnvironment>
    "#;

    fn expected_deserialized() -> ZusiEnvironmentConfig<Vec<ExampleValue>> {
        ZusiEnvironmentConfig {
            data_dir: "path/to/data_dir".into(),
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