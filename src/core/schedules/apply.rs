use thiserror::Error;
use crate::core::schedules::prepare_entries::prepare_entries;
use crate::input::schedule::{Schedule, TimeFix};
use time::{Duration, PrimitiveDateTime};
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ApplyScheduleError {
    /// occours if ankunft of FahrplanEintrag is None
    #[error("The given stop time couldn't be applied for '{betriebsstelle}'. Does the entry have both 'Ankunft' and 'Abfahrt' set?")]
    CannotApplyStopTime {
        betriebsstelle: String,
    },

    #[error("Multiple entries with time fix were found, but only one is allowed.")]
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

pub fn apply_schedule(fahrplan_eintraege: &mut Vec<FahrplanEintrag>, schedule: &Schedule) -> Result<(), ApplyScheduleError> {
    let mut prepared_entries = prepare_entries(fahrplan_eintraege, schedule);

    let ApplyScheduleState { time_fix_diff, .. } = prepared_entries.iter_mut().try_fold(
        ApplyScheduleState::new(),
        |mut state, (fahrplan_eintrag, schedule_entry)| {
            match schedule_entry.take() {
                None => {
                    fahrplan_eintrag.ankunft = fahrplan_eintrag.ankunft.map(|ankunft| ankunft + state.previous_abfahrt_change);
                    fahrplan_eintrag.abfahrt = fahrplan_eintrag.abfahrt.map(|abfahrt| abfahrt + state.previous_abfahrt_change);

                    state.previous_abfahrt = None;
                    state.previous_abfahrt_change = Duration::seconds(0);
                }
                Some(schedule_entry) => {
                    let abfahrt = fahrplan_eintrag.abfahrt.unwrap(); // fahrplan_eintrag.abfahrt.unwrap() is always Some since prepare_entries only returns those entries
                    let (ankunft, is_stop) = match fahrplan_eintrag.ankunft {
                        None => (abfahrt, false),
                        Some(ankunft) => (ankunft, true),
                    };
                    let stop_time = match (schedule_entry.stop_time, is_stop) {
                        (Some(stop_time), true) => stop_time,
                        (Some(_), false) =>
                            return Err(ApplyScheduleError::CannotApplyStopTime { betriebsstelle: fahrplan_eintrag.betriebsstelle.clone() }),
                        (None, _) => abfahrt - ankunft,
                    };
                    let new_ankunft = match state.previous_abfahrt {
                        None => ankunft,
                        Some(abfahrt) => abfahrt + schedule_entry.driving_time,
                    };
                    let new_abfahrt = new_ankunft + stop_time;

                    if is_stop {
                        fahrplan_eintrag.ankunft = Some(new_ankunft);
                    }
                    fahrplan_eintrag.abfahrt = Some(new_abfahrt);

                    state.previous_abfahrt = Some(new_abfahrt);
                    state.previous_abfahrt_change = new_abfahrt - abfahrt;

                    match (&schedule_entry.time_fix, state.time_fix_diff) {
                        (Some(TimeFix::Ankunft), None) if ankunft != new_ankunft => state.time_fix_diff = Some(ankunft - new_ankunft),
                        (Some(TimeFix::Abfahrt), None) if abfahrt != new_abfahrt => state.time_fix_diff = Some(abfahrt - new_abfahrt),
                        (Some(_), Some(_)) => return Err(ApplyScheduleError::TimeFixIsAllowedOnlyOnce),
                        (_, _) => {}
                    }
                }
            }

            Ok(state)
        }
    )?;

    if let Some(time_fix_diff) = time_fix_diff {
        for (fahrplan_eintrag, _) in prepared_entries {
            fahrplan_eintrag.ankunft = fahrplan_eintrag.ankunft.map(|ankunft| ankunft + time_fix_diff);
            fahrplan_eintrag.abfahrt = fahrplan_eintrag.abfahrt.map(|abfahrt| abfahrt + time_fix_diff);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::schedule::ScheduleEntry;
    use time::macros::datetime;
    use time::Duration;

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
                    time_fix: Some(TimeFix::Ankunft),
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

        apply_schedule(&mut modified, &schedule).unwrap();

        assert_eq!(modified, vec![
            FahrplanEintrag::builder().betriebsstelle("A".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("A".into()).abfahrt(Some(datetime!(2022-07-29 04:00:40))).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).abfahrt(Some(datetime!(2022-07-29 04:03:40))).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).ankunft(Some(datetime!(2022-07-29 04:06:40))).abfahrt(Some(datetime!(2022-07-29 04:07:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).ankunft(Some(datetime!(2022-07-29 04:12:00))).abfahrt(Some(datetime!(2022-07-29 04:13:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("E".into()).abfahrt(Some(datetime!(2022-07-29 04:16:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("F".into()).ankunft(Some(datetime!(2022-07-29 04:20:00))).abfahrt(Some(datetime!(2022-07-29 04:20:40))).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("G".into()).abfahrt(Some(datetime!(2022-07-29 04:22:40))).build(),
        ]);
    }
}