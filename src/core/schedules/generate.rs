use crate::input::schedule::{Schedule, ScheduleEntry};
use time::{Duration, PrimitiveDateTime};
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

struct GenerateScheduleState {
    entries: Vec<ScheduleEntry>,
    previous_abfahrt: Option<PrimitiveDateTime>,
}

impl GenerateScheduleState {
    fn new() -> Self {
        Self {
            entries: vec![],
            previous_abfahrt: None,
        }
    }
}

impl From<GenerateScheduleState> for Vec<ScheduleEntry> {
    fn from(value: GenerateScheduleState) -> Self {
        value.entries
    }
}

pub fn generate_schedule(fahrplan_eintraege: &Vec<FahrplanEintrag>) -> Schedule {
    Schedule {
        entries: fahrplan_eintraege
            .iter()
            .filter(|eintrag| eintrag.abfahrt.is_some())
            .fold(GenerateScheduleState::new(), |mut state, eintrag| {
                let abfahrt = eintrag.abfahrt.unwrap(); // abfahrt is always Some due to filter
                let ankunft = eintrag.ankunft.unwrap_or(abfahrt);
                let driving_time = match state.previous_abfahrt {
                    None => Duration::seconds(0),
                    Some(previous_abfahrt) => ankunft - previous_abfahrt,
                };
                state.entries.push(ScheduleEntry {
                    betriebsstelle: eintrag.betriebsstelle.clone(),
                    driving_time,
                    stop_time: if eintrag.ankunft.is_some() { Some(abfahrt - ankunft) } else { None },
                    time_fix: None,
                });
                state.previous_abfahrt = Some(abfahrt);
                state
            })
            .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_generate_schedule() {
        let input = vec![
            FahrplanEintrag::builder().betriebsstelle("A".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("A".into()).abfahrt(Some(datetime!(2022-07-29 04:03:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).abfahrt(Some(datetime!(2022-07-29 04:06:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).ankunft(Some(datetime!(2022-07-29 04:08:00))).abfahrt(Some(datetime!(2022-07-29 04:08:10))).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).ankunft(Some(datetime!(2022-07-29 04:12:00))).abfahrt(Some(datetime!(2022-07-29 04:13:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).abfahrt(Some(datetime!(2022-07-29 04:13:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).ankunft(Some(datetime!(2022-07-29 04:17:00))).abfahrt(Some(datetime!(2022-07-29 04:17:20))).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).abfahrt(Some(datetime!(2022-07-29 04:19:20))).build(),
        ];

        let expected = Schedule {
            entries: vec![
                ScheduleEntry {
                    betriebsstelle: "A".into(),
                    driving_time: Duration::seconds(0),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "B".into(),
                    driving_time: Duration::minutes(3),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "C".into(),
                    driving_time: Duration::minutes(2),
                    stop_time: Some(Duration::seconds(10)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "D".into(),
                    driving_time: Duration::minutes(3) + Duration::seconds(50),
                    stop_time: Some(Duration::minutes(1)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "E".into(),
                    driving_time: Duration::minutes(0),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "F".into(),
                    driving_time: Duration::minutes(4),
                    stop_time: Some(Duration::seconds(20)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "G".into(),
                    driving_time: Duration::minutes(2),
                    stop_time: None,
                    time_fix: None,
                },
            ],
        };

        assert_eq!(generate_schedule(&input), expected);
    }
}