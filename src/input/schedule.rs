use serde::{Deserialize, Serialize};
use serde_helpers::with::duration::duration_format;
use serde_helpers::with::duration::duration_option_format;
use time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    #[serde(rename = "ScheduleEntry")]
    pub entries: Vec<ScheduleEntry>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ScheduleEntry {
    #[serde(rename = "@betriebsstelle")]
    pub betriebsstelle: String,

    #[serde(rename = "@drivingTime", with = "duration_format")]
    pub driving_time: Duration,

    #[serde(rename = "@stopTime", with = "duration_option_format", default, skip_serializing_if = "Option::is_none")]
    pub stop_time: Option<Duration>,

    #[serde(rename = "@timeFix", default, skip_serializing_if = "Option::is_none")]
    pub time_fix: Option<TimeFix>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum TimeFix {
    #[serde(rename = "Ank")]
    Ankunft,

    #[serde(rename = "Abf")]
    Abfahrt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{de, se};
    use serde_helpers::xml::test_utils::cleanup_xml;
    use time::Duration;

    const SERIALIZED_SCHEDULE: &'static str = r#"
        <Schedule>
            <ScheduleEntry betriebsstelle="a" drivingTime="00:00:00" stopTime="00:00:50"/>
            <ScheduleEntry betriebsstelle="b" drivingTime="00:02:40" stopTime="00:00:50"/>
            <ScheduleEntry betriebsstelle="b" drivingTime="00:00:00"/>
            <ScheduleEntry betriebsstelle="c" drivingTime="00:03:10" stopTime="00:00:20" timeFix="Abf"/>
            <ScheduleEntry betriebsstelle="d" drivingTime="00:02:30" stopTime="00:00:00"/>
        </Schedule>
    "#;

    fn deserialized_schedule() -> Schedule {
        Schedule {
            entries: vec![
                ScheduleEntry {
                    betriebsstelle: "a".into(),
                    driving_time: Duration::seconds(0),
                    stop_time: Some(Duration::seconds(50)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "b".into(),
                    driving_time: Duration::minutes(2) + Duration::seconds(40),
                    stop_time: Some(Duration::seconds(50)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "b".into(),
                    driving_time: Duration::seconds(0),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "c".into(),
                    driving_time: Duration::minutes(3) + Duration::seconds(10),
                    stop_time: Some(Duration::seconds(20)),
                    time_fix: Some(TimeFix::Abfahrt),
                },
                ScheduleEntry {
                    betriebsstelle: "d".into(),
                    driving_time: Duration::minutes(2) + Duration::seconds(30),
                    stop_time: Some(Duration::seconds(0)),
                    time_fix: None,
                },
            ],
        }
    }

    #[test]
    fn test_serialize() {
        let serialized = se::to_string(&deserialized_schedule()).unwrap();
        assert_eq!(serialized, cleanup_xml(SERIALIZED_SCHEDULE.into()));
    }

    #[test]
    fn test_deserialize() {
        let deserialized: Schedule = de::from_str(SERIALIZED_SCHEDULE).unwrap();
        assert_eq!(deserialized, deserialized_schedule());
    }
}