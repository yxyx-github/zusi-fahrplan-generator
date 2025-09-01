use crate::core::fahrplan_generator::generate_zug::generate_route::ResolvedRoutePart;
use time::Duration;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeRoutePartsError {
    /// For criteria see [can_merge]
    RoutesDoNotFitTogether,
    MoreThanOneTimeFix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// For criteria see [can_merge]
pub struct RoutesCanNotBeMerged;

pub fn merge_routes(mut current: ResolvedRoutePart, mut new: ResolvedRoutePart) -> Result<ResolvedRoutePart, MergeRoutePartsError> {
    if current.has_time_fix && new.has_time_fix {
        return Err(MergeRoutePartsError::MoreThanOneTimeFix);
    }
    if can_merge(current.fahrplan_eintraege.last().unwrap(), new.fahrplan_eintraege.first().unwrap()) {
        // TODO: warn about not merge relevant differences?
        let current_last = current.fahrplan_eintraege.pop().unwrap(); // already checked by generate_route_part()
        let first_new = new.fahrplan_eintraege.first().unwrap(); // already checked by generate_route_part()
        let time_diff = get_time_diff_for_merge(&current_last, first_new).unwrap(); // already checked by can_merge()

        let (items, time_diff) = if new.has_time_fix {
            current.has_time_fix = true;
            (&mut current, -time_diff)
        } else {
            (&mut new, time_diff)
        };
        items.fahrplan_eintraege
            .iter_mut()
            .for_each(|eintrag| {
                eintrag.ankunft = eintrag.ankunft.map(|ankunft| ankunft + time_diff);
                eintrag.abfahrt = eintrag.abfahrt.map(|abfahrt| abfahrt + time_diff);
            });

        current.fahrplan_eintraege.append(&mut new.fahrplan_eintraege);
        Ok(current)
    } else {
        Err(MergeRoutePartsError::RoutesDoNotFitTogether)
    }
}

fn can_merge(first: &FahrplanEintrag, second: &FahrplanEintrag) -> bool {
    first.betriebsstelle == second.betriebsstelle &&
        first.fahrplan_signal_eintraege == second.fahrplan_signal_eintraege &&
        first.abfahrt.is_some() && second.abfahrt.is_some() &&
        first.ankunft.is_some() == second.ankunft.is_some()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NoTimes;

fn get_time_diff_for_merge(first: &FahrplanEintrag, second: &FahrplanEintrag) -> Result<Duration, NoTimes> {
    if let (Some(first), Some(second)) = (first.ankunft, second.ankunft) {
        Ok(first - second)
    } else if let (Some(first), Some(second)) = (first.abfahrt, second.abfahrt) {
        Ok(first - second)
    } else {
        Err(NoTimes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;

    #[test]
    fn test_merge_routes_by_abfahrt() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
            ],
            has_time_fix: false,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_abfahrt_current_has_time_fix() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: true,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:57:30))).build(),
            ],
            has_time_fix: true,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_abfahrt_new_has_time_fix() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:38:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:47:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
            ],
            has_time_fix: false,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:47:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:48:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:55:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft_current_has_time_fix() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: true,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:55:30))).build(),
            ],
            has_time_fix: true,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft_new_has_time_fix() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
        };
        let expected = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:40:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_cannot_merge_routes_that_do_not_fit_together() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:47:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
            ],
            has_time_fix: false,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
        };

        assert_eq!(merge_routes(current, new).unwrap_err(), MergeRoutePartsError::RoutesDoNotFitTogether);
    }

    #[test]
    fn test_cannot_merge_routes_that_both_have_time_fix() {
        let current = ResolvedRoutePart {
            aufgleis_fahrstrasse: "X -> A".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:47:20))).build(),
            ],
            has_time_fix: true,
        };
        let new = ResolvedRoutePart {
            aufgleis_fahrstrasse: "A -> B".into(),
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:47:20))).build(),
            ],
            has_time_fix: true,
        };

        assert_eq!(merge_routes(current, new).unwrap_err(), MergeRoutePartsError::MoreThanOneTimeFix);
    }

    #[test]
    fn test_can_merge() {
        assert!(can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:40))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
        ));

        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:40))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
            ]).build(),
        ));
        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
            &FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:40))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
        ));
        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
            ]).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:40))).fahrplan_signal_eintraege(vec![
                FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
            ]).build(),
        ));

        assert!(can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:10))).abfahrt(Some(datetime!(2020-09-09 07:49:30))).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:20))).abfahrt(Some(datetime!(2020-09-09 07:49:50))).build(),
        ));
        assert!(can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:50))).build(),
        ));

        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:30))).abfahrt(Some(datetime!(2020-09-09 07:49:30))).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).build(),
        ));
        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:30))).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:30))).build(),
        ));
        assert!(!can_merge(
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).build(),
            &FahrplanEintrag::builder().betriebsstelle("ADorf".into()).build(),
        ));
    }

    #[test]
    fn test_get_time_diff_for_merge() {
        assert_eq!(
            get_time_diff_for_merge(
                &FahrplanEintrag::builder().ankunft(Some(datetime!(2023-09-06 06:36:40))).abfahrt(Some(datetime!(2023-09-06 06:37:40))).build(),
                &FahrplanEintrag::builder().ankunft(Some(datetime!(2023-09-06 06:32:40))).abfahrt(Some(datetime!(2023-09-06 06:32:50))).build(),
            ).unwrap(),
            Duration::minutes(4),
        );
        assert_eq!(
            get_time_diff_for_merge(
                &FahrplanEintrag::builder().abfahrt(Some(datetime!(2023-09-06 06:37:40))).build(),
                &FahrplanEintrag::builder().abfahrt(Some(datetime!(2023-09-06 06:32:50))).build(),
            ).unwrap(),
            Duration::minutes(4) + Duration::seconds(50),
        );
        assert_eq!(
            get_time_diff_for_merge(
                &FahrplanEintrag::builder().build(),
                &FahrplanEintrag::builder().build(),
            ).unwrap_err(),
            NoTimes,
        );
    }
}