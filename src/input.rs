use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod fahrplan_config;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields, rename = "Environment")]
pub struct ZusiEnvironment<T> {
    /// Path to own data directory root
    #[serde(rename = "@basePath")]
    pub base_path: PathBuf,

    #[serde(rename = "$value")]
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::cleanup_xml;
    use quick_xml::{de, se};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct ExampleValue {
        #[serde(rename = "$value")]
        pub value: String,
    }

    const EXPECTED_SERIALIZED: &'static str = r#"
        <Environment basePath="path/to/base">
            <ExampleValue>A</ExampleValue>
            <ExampleValue>B</ExampleValue>
            <ExampleValue>C</ExampleValue>
        </Environment>
    "#;

    fn expected_deserialized() -> ZusiEnvironment<Vec<ExampleValue>> {
        ZusiEnvironment {
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
        let deserialized: ZusiEnvironment<Vec<ExampleValue>> = de::from_str(EXPECTED_SERIALIZED).unwrap();
        assert_eq!(deserialized, expected_deserialized());
    }
}