use serde::{Deserialize, Serialize};
use serde_helpers::with::date_time::date_time_format;
use serde_helpers::with::duration::duration_format;
use std::ops::Not;
use std::path::PathBuf;
use time::{Duration, PrimitiveDateTime};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields, rename = "Fahrplan")]
pub struct FahrplanConfig {
    /// Path where to place the generated .fpn file
    #[serde(rename = "@generateAt")]
    pub generate_at: PathBuf,

    /// Path to .fpn file to use for Streckenmodule and UTM data
    #[serde(rename = "@generateFrom")]
    pub generate_from: PathBuf,

    #[serde(rename = "Train", default)]
    pub trains: Vec<TrainConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrainConfig {
    #[serde(rename = "@nummer")]
    pub nummer: String,

    #[serde(rename = "@gattung")]
    pub gattung: String,

    #[serde(rename = "Route")]
    pub route: RouteConfig,

    #[serde(rename = "RollingStock")]
    pub rolling_stock: RollingStock,

    #[serde(rename = "CopyDelay", default, skip_serializing_if = "Option::is_none")]
    pub copy_delay_config: Option<CopyDelayConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RouteConfig {
    #[serde(rename = "$value")]
    pub parts: Vec<RoutePart>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RoutePart {
    #[serde(rename = "$value")]
    pub source: RoutePartSource,

    #[serde(rename = "@overrideMetaData", default, skip_serializing_if = "<&bool>::not")]
    pub override_meta_data: bool, // TODO: remove, instead add meta data source to TrainConfig directly

    #[serde(rename = "TimeFix", default, skip_serializing_if = "Option::is_none")]
    pub time_fix: Option<RouteTimeFix>,

    #[serde(rename = "ApplySchedule", default, skip_serializing_if = "Option::is_none")]
    pub apply_schedule: Option<ApplySchedule>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RoutePartSource {
    TrainFileByPath {
        #[serde(rename = "@path")]
        path: PathBuf,
    },
    TrainConfigByNummer {
        #[serde(rename = "@nummer")]
        nummer: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RollingStock {
    #[serde(rename = "@path")]
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RouteTimeFix {
    #[serde(rename = "@type")]
    pub fix_type: RouteTimeFixType,

    #[serde(rename = "@value", with = "date_time_format")]
    pub value: PrimitiveDateTime,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RouteTimeFixType {
    StartAbf,
    EndAnk,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ApplySchedule {
    #[serde(rename = "@path")]
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyDelayConfig {
    #[serde(rename = "CopyDelayTask")]
    pub tasks: Vec<CopyDelayTask>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyDelayTask {
    #[serde(rename = "@delay", with = "duration_format")]
    pub delay: Duration,

    #[serde(rename = "@count")]
    pub count: u32,

    #[serde(rename = "@increment")]
    pub increment: i32,

    #[serde(rename = "RollingStock", default, skip_serializing_if = "Option::is_none")]
    pub custom_rolling_stock: Option<RollingStock>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ZusiEnvironmentConfig;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;
    use time::macros::datetime;

    const EXPECTED_SERIALIZED: &'static str = r#"
        <ZusiEnvironment basePath="path/to/Zusi3User">
            <Fahrplan generateAt="./path/to/destination.fpn" generateFrom="./path/to/template.fpn">
                <Train nummer="20000" gattung="RB">
                    <Route>
                        <RoutePart overrideMetaData="true">
                            <TrainFileByPath path="./path/to/route-part.trn"/>
                            <TimeFix type="StartAbf" value="2023-02-01 13:50:20"/>
                            <ApplySchedule path="./path/to/a.schedule.xml"/>
                        </RoutePart>
                        <RoutePart>
                            <TrainConfigByNummer nummer="10000"/>
                        </RoutePart>
                    </Route>
                    <RollingStock path="./path/to/rolling-stock.trn"/>
                    <CopyDelay>
                        <CopyDelayTask delay="03:00:00" count="1" increment="6"/>
                        <CopyDelayTask delay="02:00:00" count="3" increment="2">
                            <RollingStock path="./path/to/rolling-stock.trn"/>
                        </CopyDelayTask>
                    </CopyDelay>
                </Train>
            </Fahrplan>
        </ZusiEnvironment>
    "#;

    fn expected_deserialized() -> ZusiEnvironmentConfig<FahrplanConfig> {
        ZusiEnvironmentConfig {
            base_path: "path/to/Zusi3User".into(),
            value: FahrplanConfig {
                generate_at: "./path/to/destination.fpn".into(),
                generate_from: "./path/to/template.fpn".into(),
                trains: vec![
                    TrainConfig {
                        nummer: "20000".into(),
                        gattung: "RB".into(),
                        route: RouteConfig {
                            parts: vec![
                                RoutePart {
                                    source: RoutePartSource::TrainFileByPath { path: "./path/to/route-part.trn".into() },
                                    override_meta_data: true,
                                    time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2023-02-01 13:50:20) }),
                                    apply_schedule: Some(ApplySchedule { path: "./path/to/a.schedule.xml".into() }),
                                },
                                RoutePart {
                                    source: RoutePartSource::TrainConfigByNummer { nummer: "10000".into() },
                                    override_meta_data: false,
                                    time_fix: None,
                                    apply_schedule: None,
                                },
                            ],
                        },
                        rolling_stock: RollingStock { path: "./path/to/rolling-stock.trn".into() },
                        copy_delay_config: Some(CopyDelayConfig {
                            tasks: vec![
                                CopyDelayTask {
                                    delay: Duration::hours(3),
                                    count: 1,
                                    increment: 6,
                                    custom_rolling_stock: None,
                                },
                                CopyDelayTask {
                                    delay: Duration::hours(2),
                                    count: 3,
                                    increment: 2,
                                    custom_rolling_stock: Some(RollingStock { path: "./path/to/rolling-stock.trn".into() }),
                                },
                            ],
                        }),
                    },
                ],
            },
        }
    }

    #[test]
    fn test_serialize() {
        let serialized = se::to_string(&expected_deserialized()).unwrap();
        assert_eq!(serialized, cleanup_xml(EXPECTED_SERIALIZED.into()));
    }

    #[test]
    fn test_deserialize() {
        let deserialized: ZusiEnvironmentConfig<FahrplanConfig> = de::from_str(EXPECTED_SERIALIZED).unwrap();
        assert_eq!(deserialized, expected_deserialized());
    }
}