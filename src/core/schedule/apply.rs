use time::{Duration, PrimitiveDateTime};
use crate::core::schedule::prepare_entries::prepare_entries;
use crate::input::schedule::{Schedule, ScheduleEntry, TimeFix};
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyScheduleError {
    /// occours if ankunft of FahrplanEintrag is None
    CannotApplyStopTime {
        betriebsstelle: String,
    },
    TimeFixIsAllowedOnlyOnce,
}

struct ApplyScheduleState {
    previous_abfahrt: Option<PrimitiveDateTime>,
    previous_abfahrt_change: Duration,
    time_fix_diff: Option<Duration>,
}

impl ApplyScheduleState {
    fn new() -> Self {
        Self {
            previous_abfahrt: None,
            previous_abfahrt_change: Duration::seconds(0),
            time_fix_diff: None,
        }
    }
}

pub fn apply(fahrplan_eintraege: &mut Vec<FahrplanEintrag>, schedule: &Schedule) -> Result<(), ApplyScheduleError> {
    prepare_entries(fahrplan_eintraege, schedule).into_iter().try_fold(
        ApplyScheduleState::new(),
        |mut state, (fahrplan_eintrag, schedule_entry)| {
            let abfahrt = fahrplan_eintrag.abfahrt.unwrap(); // fahrplan_eintrag.abfahrt.unwrap() is always Some since prepare_entries only returns those entries
            let (ankunft, is_stop) = match fahrplan_eintrag.ankunft {
                None => (abfahrt, false),
                Some(ankunft) => (ankunft, true),
            };
            let stop_time = match (schedule_entry, is_stop) {
                (Some(ScheduleEntry { stop_time: Some(stop_time), .. }), true) => *stop_time,
                (Some(ScheduleEntry { stop_time: Some(_), .. }), false) =>
                    return Err(ApplyScheduleError::CannotApplyStopTime { betriebsstelle: fahrplan_eintrag.betriebsstelle.clone() }),
                (_, _) => abfahrt - ankunft,
            };
            let new_ankunft = match state.previous_abfahrt {
                None => ankunft,
                Some(abfahrt) => schedule_entry
                    .map(|entry| abfahrt + entry.driving_time)
                    .unwrap_or(ankunft + state.previous_abfahrt_change),
            };
            let new_abfahrt = new_ankunft + stop_time;

            if is_stop {
                fahrplan_eintrag.ankunft = Some(new_ankunft);
            }
            fahrplan_eintrag.abfahrt = Some(new_abfahrt);

            if schedule_entry.is_some() {
                state.previous_abfahrt = Some(new_abfahrt);
                state.previous_abfahrt_change = new_abfahrt - abfahrt;
            } else {
                state.previous_abfahrt = None;
                state.previous_abfahrt_change = Duration::seconds(0);
            }

            match (schedule_entry, state.time_fix_diff) {
                (Some(ScheduleEntry { time_fix: Some(TimeFix::Ankunft), .. }), None) => state.time_fix_diff = Some(ankunft - new_ankunft),
                (Some(ScheduleEntry { time_fix: Some(TimeFix::Abfahrt), .. }), None) => state.time_fix_diff = Some(abfahrt - new_abfahrt),
                (Some(ScheduleEntry { time_fix: Some(_), .. }), Some(_)) => return Err(ApplyScheduleError::TimeFixIsAllowedOnlyOnce),
                (_, _) => {}
            }

            Ok(state)
        }
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use time::Duration;
    use time::macros::datetime;
    use crate::input::schedule::ScheduleEntry;
    use super::*;

    #[test]
    fn test_apply() {
        let fahrplan_eintraege = vec![
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

        let schedule = Schedule {
            entries: vec![
                ScheduleEntry {
                    betriebsstelle: "B".into(),
                    driving_time: Duration::minutes(0),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "C".into(),
                    driving_time: Duration::minutes(3),
                    stop_time: Some(Duration::seconds(20)),
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "D".into(),
                    driving_time: Duration::minutes(5),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "E".into(),
                    driving_time: Duration::minutes(3),
                    stop_time: None,
                    time_fix: None,
                },
                ScheduleEntry {
                    betriebsstelle: "F".into(),
                    driving_time: Duration::minutes(4),
                    stop_time: Some(Duration::seconds(40)),
                    time_fix: None,
                },
            ],
        };

        let mut modified = fahrplan_eintraege.clone();

        apply(&mut modified, &schedule).unwrap();

        for entry in modified.iter() {
            println!("{}: {:?} - {:?}", entry.betriebsstelle, entry.ankunft, entry.abfahrt);
        }

        assert_eq!(modified, vec![
            FahrplanEintrag::builder().betriebsstelle("A".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("A".into()).abfahrt(Some(datetime!(2022-07-29 04:03:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).abfahrt(Some(datetime!(2022-07-29 04:06:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).ankunft(Some(datetime!(2022-07-29 04:09:00))).abfahrt(Some(datetime!(2022-07-29 04:09:20))).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).ankunft(Some(datetime!(2022-07-29 04:14:20))).abfahrt(Some(datetime!(2022-07-29 04:15:20))).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).abfahrt(Some(datetime!(2022-07-29 04:18:20))).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).ankunft(Some(datetime!(2022-07-29 04:22:20))).abfahrt(Some(datetime!(2022-07-29 04:23:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).abfahrt(Some(datetime!(2022-07-29 04:25:00))).build(),
        ]);
    }
}