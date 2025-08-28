use crate::core::schedule::longest_common_coherent_subsequence::longest_common_coherent_subsequence;
use crate::input::schedule::{Schedule, ScheduleEntry};
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

pub fn prepare_entries<'f, 's>(mut fahrplan_eintraege: &'f mut Vec<FahrplanEintrag>, schedule: &'s Schedule) -> Vec<(&'f mut FahrplanEintrag, Option<&'s ScheduleEntry>)> {
    let mut fahrplan_eintraege: Vec<&'f mut FahrplanEintrag> = fahrplan_eintraege
        .iter_mut()
        .filter(|eintrag| eintrag.abfahrt.is_some())
        .collect();

    let fahrplan_betriebsstellen: Vec<&str> = fahrplan_eintraege
        .iter()
        .map(|eintrag| eintrag.betriebsstelle.as_str())
        .collect();
    let schedule_betriebsstellen: Vec<&str> = schedule.entries
        .iter()
        .map(|entry| entry.betriebsstelle.as_str())
        .collect();

    let lccm = longest_common_coherent_subsequence(
        fahrplan_betriebsstellen,
        schedule_betriebsstellen
    );

    let mut prepared_schedule_entries: Vec<Option<&ScheduleEntry>> = (0..lccm.sec1_start)
        .into_iter()
        .map(|_| None)
        .collect();
    prepared_schedule_entries.append(
        &mut schedule.entries[lccm.sec2_start..lccm.sec2_start + lccm.len]
            .iter()
            .map(|s| Some(s))
            .collect()
    );
    prepared_schedule_entries.append(
        &mut (lccm.sec1_start + lccm.len..fahrplan_eintraege.len())
            .into_iter()
            .map(|_| None)
            .collect()
    );
    fahrplan_eintraege.into_iter().zip(prepared_schedule_entries.into_iter()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use time::Duration;

    #[test]
    fn test_prepare_entries() {
        let mut fahrplan_eintraege = vec![
            FahrplanEintrag::builder().betriebsstelle("A".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("A".into()).abfahrt(Some(datetime!(2022-07-29 04:03:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("B".into()).abfahrt(Some(datetime!(2022-07-29 04:06:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("C".into()).abfahrt(Some(datetime!(2022-07-29 04:08:00))).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).build(),
            FahrplanEintrag::builder().betriebsstelle("D".into()).abfahrt(Some(datetime!(2022-07-29 04:12:00))).build(),
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
                    stop_time: None,
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
                    driving_time: Duration::minutes(7),
                    stop_time: None,
                    time_fix: None,
                },
            ],
        };

        assert_eq!(
            prepare_entries(&mut fahrplan_eintraege, &schedule)
                .into_iter()
                .map(|(f, s)| (f.clone(), s.clone()))
                .collect::<Vec<_>>(),
            vec![
                (fahrplan_eintraege[1].clone(), None),
                (fahrplan_eintraege[3].clone(), Some(&schedule.entries[0].clone())),
                (fahrplan_eintraege[5].clone(), Some(&schedule.entries[1].clone())),
                (fahrplan_eintraege[7].clone(), Some(&schedule.entries[2].clone())),
            ],
        );
    }
}