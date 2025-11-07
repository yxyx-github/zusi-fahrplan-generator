pub mod non_default_fahrzeug_verband_aktion;

use crate::input::apply_schedule::ApplySchedule;
use crate::input::copy_delay_config::CopyDelayConfig;
use crate::input::fahrplan_config::non_default_fahrzeug_verband_aktion::NonDefaultFahrzeugVerbandAktion;
use crate::input::rolling_stock_config::RollingStockConfig;
use serde::{Deserialize, Serialize};
use serde_helpers::default::IsDefault;
use serde_helpers::with::bool_as_int::bool_as_int_format;
use serde_helpers::with::date_time::date_time_format;
use std::path::PathBuf;
use time::PrimitiveDateTime;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields, rename = "Fahrplan")]
pub struct FahrplanConfig {
    /// Path where to place the generated .fpn file
    #[serde(rename = "@generateAt")]
    pub generate_at: PathBuf,

    /// Path to .fpn file to use for Streckenmodule and UTM data
    #[serde(rename = "@generateFrom")]
    pub generate_from: PathBuf,

    #[serde(rename = "Zug", default)]
    pub zuege: Vec<ZugConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ZugConfig {
    #[serde(rename = "@nummer")]
    pub nummer: String,

    #[serde(rename = "@gattung")]
    pub gattung: String,

    #[serde(rename = "@zuglauf", default, skip_serializing_if = "IsDefault::is_default")]
    pub zuglauf: String,

    #[serde(rename = "@fahrplanGruppe", default, skip_serializing_if = "IsDefault::is_default")]
    pub fahrplan_gruppe: String,

    #[serde(rename = "MetaData", default, skip_serializing_if = "Option::is_none")]
    pub meta_data: Option<MetaDataConfig>,

    #[serde(rename = "Route")]
    pub route: RouteConfig,

    #[serde(rename = "RollingStock")]
    pub rolling_stock: RollingStockConfig,

    #[serde(rename = "CopyDelay", default, skip_serializing_if = "Option::is_none")]
    pub copy_delay_config: Option<CopyDelayConfig>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetaDataConfig {
    #[serde(rename = "@path")]
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RouteConfig {
    #[serde(rename = "$value")]
    pub parts: Vec<RoutePart>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RoutePart {
    #[serde(rename = "$value")]
    pub source: RoutePartSource,

    #[serde(rename = "StartFahrzeugVerbandAktion", default, skip_serializing_if = "Option::is_none")]
    pub start_fahrzeug_verband_aktion: Option<StartFahrzeugVerbandAktion>,

    #[serde(rename = "TimeFix", default, skip_serializing_if = "Option::is_none")]
    pub time_fix: Option<RouteTimeFix>,

    #[serde(rename = "ApplySchedule", default, skip_serializing_if = "Option::is_none")]
    pub apply_schedule: Option<ApplySchedule>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StartFahrzeugVerbandAktion {
    #[serde(rename = "@aktion")]
    pub aktion: NonDefaultFahrzeugVerbandAktion,

    #[serde(rename = "@wendeSignal", with = "bool_as_int_format", default, skip_serializing_if = "IsDefault::is_default")]
    pub wende_signal: bool,

    #[serde(rename = "@wendeSignalAbstand", default, skip_serializing_if = "IsDefault::is_default")]
    pub wende_signal_abstand: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RouteTimeFix {
    #[serde(rename = "@type")]
    pub fix_type: RouteTimeFixType,

    #[serde(rename = "@value", with = "date_time_format")]
    pub value: PrimitiveDateTime,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RouteTimeFixType {
    StartAbf,
    EndAnk,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::copy_delay_config::CopyDelayTask;
    use crate::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;
    use time::macros::datetime;
    use time::Duration;

    const EXPECTED_SERIALIZED: &'static str = r#"
        <ZusiEnvironment dataDir="path/to/Zusi3User">
            <Fahrplan generateAt="./path/to/destination.fpn" generateFrom="./path/to/template.fpn">
                <Zug nummer="20000" gattung="RB" zuglauf="ADorf - BDorf" fahrplanGruppe="A - B">
                    <MetaData path="./path/to/meta-data.trn"/>
                    <Route>
                        <RoutePart>
                            <TrainFileByPath path="./path/to/route-part.trn"/>
                            <TimeFix type="StartAbf" value="2023-02-01 13:50:20"/>
                            <ApplySchedule path="./path/to/a.schedule.xml"/>
                        </RoutePart>
                        <RoutePart>
                            <TrainFileByPath path="./path/to/route-part.trn"/>
                            <StartFahrzeugVerbandAktion aktion="2" wendeSignal="1" wendeSignalAbstand="200"/>
                            <ApplySchedule path="./path/to/b.schedule.xml" firstStopTime="00:04:00" lastStopTime="00:40:00"/>
                        </RoutePart>
                        <RoutePart>
                            <TrainConfigByNummer nummer="10000"/>
                        </RoutePart>
                    </Route>
                    <RollingStock path="./path/to/rolling-stock.trn"/>
                    <CopyDelay>
                        <CopyDelayTask delay="03:00:00" increment="6" count="1"/>
                        <CopyDelayTask delay="02:00:00" increment="2" count="3">
                            <RollingStock path="./path/to/rolling-stock.trn"/>
                        </CopyDelayTask>
                    </CopyDelay>
                </Zug>
                <Zug nummer="30000" gattung="RE">
                    <Route>
                        <RoutePart>
                            <TrainFileByPath path="./path/to/route-part.trn"/>
                        </RoutePart>
                    </Route>
                    <RollingStock path="./path/to/rolling-stock.trn"/>
                </Zug>
            </Fahrplan>
        </ZusiEnvironment>
    "#;

    fn expected_deserialized() -> ZusiEnvironmentConfig<FahrplanConfig> {
        ZusiEnvironmentConfig {
            data_dir: "path/to/Zusi3User".into(),
            value: FahrplanConfig {
                generate_at: "./path/to/destination.fpn".into(),
                generate_from: "./path/to/template.fpn".into(),
                zuege: vec![
                    ZugConfig {
                        nummer: "20000".into(),
                        gattung: "RB".into(),
                        zuglauf: "ADorf - BDorf".into(),
                        fahrplan_gruppe: "A - B".into(),
                        meta_data: Some(MetaDataConfig {
                            path: "./path/to/meta-data.trn".into(),
                        }),
                        route: RouteConfig {
                            parts: vec![
                                RoutePart {
                                    source: RoutePartSource::TrainFileByPath { path: "./path/to/route-part.trn".into() },
                                    start_fahrzeug_verband_aktion: None,
                                    time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2023-02-01 13:50:20) }),
                                    apply_schedule: Some(ApplySchedule {
                                        path: "./path/to/a.schedule.xml".into(),
                                        first_stop_time: None,
                                        last_stop_time: None,
                                        modifications: vec![],
                                    }),
                                },
                                RoutePart {
                                    source: RoutePartSource::TrainFileByPath { path: "./path/to/route-part.trn".into() },
                                    start_fahrzeug_verband_aktion: Some(StartFahrzeugVerbandAktion {
                                        aktion: NonDefaultFahrzeugVerbandAktion::Fueherstandswechsel,
                                        wende_signal: true,
                                        wende_signal_abstand: 200.,
                                    }),
                                    time_fix: None,
                                    apply_schedule: Some(ApplySchedule {
                                        path: "./path/to/b.schedule.xml".into(),
                                        first_stop_time: Some(Duration::minutes(4)),
                                        last_stop_time: Some(Duration::minutes(40)),
                                        modifications: vec![],
                                    }),
                                },
                                RoutePart {
                                    source: RoutePartSource::TrainConfigByNummer { nummer: "10000".into() },
                                    start_fahrzeug_verband_aktion: None,
                                    time_fix: None,
                                    apply_schedule: None,
                                },
                            ],
                        },
                        rolling_stock: RollingStockConfig { path: "./path/to/rolling-stock.trn".into() },
                        copy_delay_config: Some(CopyDelayConfig {
                            tasks: vec![
                                CopyDelayTask {
                                    delay: Duration::hours(3),
                                    first_delay: None,
                                    increment: 6,
                                    first_increment: None,
                                    count: 1,
                                    custom_rolling_stock: None,
                                },
                                CopyDelayTask {
                                    delay: Duration::hours(2),
                                    first_delay: None,
                                    increment: 2,
                                    first_increment: None,
                                    count: 3,
                                    custom_rolling_stock: Some(RollingStockConfig { path: "./path/to/rolling-stock.trn".into() }),
                                },
                            ],
                        }),
                    },
                    ZugConfig {
                        nummer: "30000".into(),
                        gattung: "RE".into(),
                        zuglauf: "".into(),
                        fahrplan_gruppe: "".into(),
                        meta_data: None,
                        route: RouteConfig {
                            parts: vec![
                                RoutePart {
                                    source: RoutePartSource::TrainFileByPath { path: "./path/to/route-part.trn".into() },
                                    start_fahrzeug_verband_aktion: None,
                                    time_fix: None,
                                    apply_schedule: None,
                                },
                            ],
                        },
                        rolling_stock: RollingStockConfig { path: "./path/to/rolling-stock.trn".into() },
                        copy_delay_config: None,
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