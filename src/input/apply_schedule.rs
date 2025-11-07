use serde_helpers::with::duration::duration_format;
use serde_helpers::default::IsDefault;
use serde::{Deserialize, Serialize};
use serde_helpers::with::duration::duration_option_format;
use std::path::PathBuf;
use time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ApplySchedule {
    #[serde(rename = "@path")]
    pub path: PathBuf,

    #[serde(rename = "@firstStopTime", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub first_stop_time: Option<Duration>,

    #[serde(rename = "@lastStopTime", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub last_stop_time: Option<Duration>,

    #[serde(rename = "ModificationBetween", default)]
    pub modifications: Vec<ScheduleModificationBetween>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScheduleModificationBetween {
    #[serde(rename = "@first")]
    pub first_betriebsstelle: String,

    #[serde(rename = "@last")]
    pub last_betriebsstelle: String,

    #[serde(rename = "$value")]
    pub modification: ScheduleModification,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ScheduleModification {
    RecalculateDrivingTimes(RecalculateDrivingTimes),
    SetTotalDrivingTime(SetTotalDrivingTime),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RecalculateDrivingTimes {
    #[serde(rename = "@pass", default, skip_serializing_if = "IsDefault::is_default")]
    pub pass: String,

    #[serde(rename = "@arrival")]
    pub arrival: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SetTotalDrivingTime {
    #[serde(rename = "@time", with = "duration_format")]
    pub total_driving_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;
    use time::Duration;

    const EXPECTED_SERIALIZED: &'static str = r#"
        <ApplySchedule path="./path/to/b.schedule.xml" firstStopTime="00:04:00" lastStopTime="00:40:00">
            <ModificationBetween first="ADorf" last="BDorf">
                <RecalculateDrivingTimes pass="formula a" arrival="formula b"/>
            </ModificationBetween>
            <ModificationBetween first="CDorf" last="DDorf">
                <SetTotalDrivingTime time="01:29:07"/>
            </ModificationBetween>
        </ApplySchedule>
    "#;

    fn expected_deserialized() -> ApplySchedule {
        ApplySchedule {
            path: "./path/to/b.schedule.xml".into(),
            first_stop_time: Some(Duration::minutes(4)),
            last_stop_time: Some(Duration::minutes(40)),
            modifications: vec![
                ScheduleModificationBetween {
                    first_betriebsstelle: "ADorf".into(),
                    last_betriebsstelle: "BDorf".into(),
                    modification: ScheduleModification::RecalculateDrivingTimes(RecalculateDrivingTimes {
                        pass: "formula a".into(),
                        arrival: "formula b".into(),
                    }),
                },
                ScheduleModificationBetween {
                    first_betriebsstelle: "CDorf".into(),
                    last_betriebsstelle: "DDorf".into(),
                    modification: ScheduleModification::SetTotalDrivingTime(SetTotalDrivingTime {
                        total_driving_time: Duration::hours(1) + Duration::minutes(29) + Duration::seconds(7),
                    }),
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
        let deserialized: ApplySchedule = de::from_str(EXPECTED_SERIALIZED).unwrap();
        assert_eq!(deserialized, expected_deserialized());
    }
}