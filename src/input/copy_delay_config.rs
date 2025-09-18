use crate::input::rolling_stock_config::RollingStockConfig;
use serde::{Deserialize, Serialize};
use serde_helpers::with::duration::duration_format;
use serde_helpers::with::duration::duration_option_format;
use time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields, rename = "CopyDelay")]
pub struct CopyDelayConfig {
    #[serde(rename = "CopyDelayTask")]
    pub tasks: Vec<CopyDelayTask>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyDelayTask {
    #[serde(rename = "@delay", with = "duration_format")]
    pub delay: Duration,

    #[serde(rename = "@firstDelay", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub first_delay: Option<Duration>,

    #[serde(rename = "@increment")]
    pub increment: i32,

    #[serde(rename = "@firstIncrement", default, skip_serializing_if = "Option::is_none")]
    pub first_increment: Option<i32>,

    #[serde(rename = "@count")]
    pub count: u32,

    #[serde(rename = "RollingStock", default, skip_serializing_if = "Option::is_none")]
    pub custom_rolling_stock: Option<RollingStockConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::copy_delay_config::CopyDelayTask;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;
    use time::Duration;

    const EXPECTED_SERIALIZED: &'static str = r#"
        <CopyDelay>
            <CopyDelayTask delay="04:00:00" firstDelay="02:00:00" increment="9" firstIncrement="4" count="2"/>
            <CopyDelayTask delay="01:00:00" increment="2" count="7">
                <RollingStock path="./path/to/rolling-stock.trn"/>
            </CopyDelayTask>
        </CopyDelay>
    "#;

    fn expected_deserialized() -> CopyDelayConfig {
        CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(4),
                    first_delay: Some(Duration::hours(2)),
                    increment: 9,
                    first_increment: Some(4),
                    count: 2,
                    custom_rolling_stock: None,
                },
                CopyDelayTask {
                    delay: Duration::hours(1),
                    first_delay: None,
                    increment: 2,
                    first_increment: None,
                    count: 7,
                    custom_rolling_stock: Some(RollingStockConfig { path: "./path/to/rolling-stock.trn".into() }),
                },
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
        let deserialized: CopyDelayConfig = de::from_str(EXPECTED_SERIALIZED).unwrap();
        assert_eq!(deserialized, expected_deserialized());
    }
}