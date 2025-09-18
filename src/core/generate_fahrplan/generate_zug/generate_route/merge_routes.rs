mod concat_buchfahrplaene;

use crate::core::generate_fahrplan::generate_zug::generate_route::merge_routes::concat_buchfahrplaene::{concat_buchfahrplaene, ConcatBuchfahrplaeneError};
use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::ResolvedRoutePart;
use crate::core::lib::helpers::delay_fahrplan_eintraege;
use crate::core::schedules::update_buchfahrplan::{update_buchfahrplan, UpdateBuchfahrplanError};
use thiserror::Error;
use time::Duration;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MergeRoutePartsError {
    /// For criteria see [can_merge]
    #[error("The route parts aren't consecutive. The last entry of the previous part must match the first entry of the next part in some criteria: 'Betriebsstelle' must be equal, 'Abfahrt' needs to be set for both entries, 'Ankunft' must be set for either both or none of these entries.")]
    NonConsecutiveRouteParts,

    #[error("Multiple route parts with time fix were found, but only one is allowed.")]
    MoreThanOneTimeFix,

    #[error("Couldn't merge 'Buchfahrplaene': {0}")]
    MergeBuchfahrplaeneError(#[from] MergeBuchfahrplaeneError)
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MergeBuchfahrplaeneError {
    #[error("{error}")]
    ConcatBuchfahrplaeneError {
        #[from]
        error: ConcatBuchfahrplaeneError,
    },

    #[error("{error}")]
    UpdateBuchfahrplaeneError {
        #[from]
        error: UpdateBuchfahrplanError,
    },
}

impl From<ConcatBuchfahrplaeneError> for MergeRoutePartsError {
    fn from(error: ConcatBuchfahrplaeneError) -> Self {
        MergeRoutePartsError::MergeBuchfahrplaeneError(MergeBuchfahrplaeneError::ConcatBuchfahrplaeneError { error })
    }
}

impl From<UpdateBuchfahrplanError> for MergeRoutePartsError {
    fn from(error: UpdateBuchfahrplanError) -> Self {
        MergeRoutePartsError::MergeBuchfahrplaeneError(MergeBuchfahrplaeneError::UpdateBuchfahrplaeneError { error })
    }
}

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
        delay_fahrplan_eintraege(&mut items.fahrplan_eintraege, time_diff);

        current.fahrplan_eintraege.append(&mut new.fahrplan_eintraege);
        
        if !current.fahrplan_zeilen.is_empty() && !new.fahrplan_zeilen.is_empty() {
            current.fahrplan_zeilen = concat_buchfahrplaene(current.fahrplan_zeilen, new.fahrplan_zeilen, &current_last.betriebsstelle)?;
            update_buchfahrplan(&current.fahrplan_eintraege, &mut current.fahrplan_zeilen)?;
        }
        if new.start_data.gnt_spalte.is_some_and(|gnt_spalte| gnt_spalte) {
            current.start_data.gnt_spalte = Some(true);
        }
        if new.mindest_bremshundertstel > current.mindest_bremshundertstel {
            current.mindest_bremshundertstel = new.mindest_bremshundertstel;
        }
        
        Ok(current)
    } else {
        Err(MergeRoutePartsError::NonConsecutiveRouteParts)
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
    use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::RouteStartData;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_km::FahrplanKm;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name_rechts::FahrplanNameRechts;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_signal_typ::FahrplanSignalTyp;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_v_max::FahrplanVMax;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::standort_modus::StandortModus;

    #[test]
    fn test_merge_routes_by_abfahrt() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
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
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 1.6,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 1.8,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
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
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 1.8,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_abfahrt_current_has_time_fix() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:57:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_abfahrt_new_has_time_fix() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:38:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
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
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
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
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft_current_has_time_fix() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:48:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:55:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_ankunft_new_has_time_fix() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:39:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:47:20))).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("XDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:40:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:48:20))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_cannot_merge_non_consecutive_routes() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
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
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    FahrplanSignalEintrag::builder().fahrplan_signal("B".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
                FahrplanEintrag::builder().betriebsstelle("CDorf".into()).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap_err(), MergeRoutePartsError::NonConsecutiveRouteParts);
    }

    #[test]
    fn test_cannot_merge_routes_that_both_have_time_fix() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:47:20))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:47:20))).build(),
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap_err(), MergeRoutePartsError::MoreThanOneTimeFix);
    }

    #[test]
    fn test_merge_routes_by_ankunft_with_buchfahrplan() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:20))).abfahrt(Some(datetime!(2020-09-09 07:59:40))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(31660.822)
                    .fahrplan_km(Some(FahrplanKm::builder().km(11.5721).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 08:49:00))).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(3435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(4435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(14.3433).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2020-09-09 07:49:20))).abfahrt(Some(datetime!(2020-09-09 07:49:40))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:56:50))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(31660.822)
                    .fahrplan_km(Some(FahrplanKm::builder().km(11.5721).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:49:10)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2020-09-09 07:49:20)).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:49:40)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(34435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(14.3433).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:56:50)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_merge_routes_by_abfahrt_with_buchfahrplan() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(31660.822)
                    .fahrplan_km(Some(FahrplanKm::builder().km(11.5721).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 1.8,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(3435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(4435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(14.3433).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 1.7,
        };
        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 07:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(31660.822)
                    .fahrplan_km(Some(FahrplanKm::builder().km(11.5721).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:49:10)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:49:20)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(34435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(14.3433).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2020-09-09 07:56:30)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 1.8,
        };

        assert_eq!(merge_routes(current, new).unwrap(), expected);
    }

    #[test]
    fn test_cannot_merge_routes_by_abfahrt_with_non_consecutive_buchfahrplan() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(31660.822)
                    .fahrplan_km(Some(FahrplanKm::builder().km(11.5721).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(212.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(212.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(3435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(213.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(4435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(214.3433).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(
            merge_routes(current, new).unwrap_err(),
            MergeRoutePartsError::MergeBuchfahrplaeneError(MergeBuchfahrplaeneError::ConcatBuchfahrplaeneError {
                error: ConcatBuchfahrplaeneError::NonConsecutiveBuchfahrplaene,
            }),
        );
    }

    #[test]
    fn test_cannot_merge_routes_by_abfahrt_with_invalid_buchfahrplan() {
        let current = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "X -> A".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:10))).fahrplan_signal_eintraege(vec![
                    FahrplanSignalEintrag::builder().fahrplan_signal("E".into()).build(),
                ]).build(),
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 07:49:20))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(32883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(33435.87)
                    .fahrplan_km(Some(FahrplanKm::builder().km(13.3433).build()))
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };
        let new = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "A -> B".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder().betriebsstelle("ADorf".into()).abfahrt(Some(datetime!(2020-09-09 08:49:20))).build(),
                FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2020-09-09 08:56:30))).build(),
            ],
            has_time_fix: false,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2660.822)
                    .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.5721).build()))
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_laufweg(2883.34)
                    .fahrplan_km(Some(FahrplanKm::builder().km(12.7907).build()))
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };

        assert_eq!(
            merge_routes(current, new).unwrap_err(),
            MergeRoutePartsError::MergeBuchfahrplaeneError(MergeBuchfahrplaeneError::UpdateBuchfahrplaeneError {
                error: UpdateBuchfahrplanError::InvalidLen,
            }),
        );
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