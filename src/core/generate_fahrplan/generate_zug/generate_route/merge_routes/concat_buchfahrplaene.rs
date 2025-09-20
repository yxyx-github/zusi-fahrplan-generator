use std::collections::VecDeque;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConcatBuchfahrplaeneError {
    /// For criteria see [can_concat]
    #[error("The 'Buchfahrplaene' aren't consecutive. The last entry representing a 'Betriebsstelle' of the previous 'Buchfahrplan' must match the first entry representing a 'Betriebsstelle' of the next 'Buchfahrplan' in some criteria: 'Km' and 'Betriebsstelle' must be equal, at least one entry needs to have 'Abfahrt' set.")]
    NonConsecutiveBuchfahrplaene,
}

pub fn concat_buchfahrplaene(mut current: Vec<FahrplanZeile>, new: Vec<FahrplanZeile>, betriebsstelle: &str) -> Result<Vec<FahrplanZeile>, ConcatBuchfahrplaeneError> {
    let mut new: VecDeque<_> = new.into();
    while let Some(zeile) = current.last() {
        match zeile {
            FahrplanZeile { fahrplan_name: Some(FahrplanName { fahrplan_name_text, .. }),.. } if fahrplan_name_text == betriebsstelle =>
                break,
            _ => {
                current.pop();
            }
        }
    }
    while let Some(zeile) = new.front() {
        match zeile {
            FahrplanZeile { fahrplan_name: Some(FahrplanName { fahrplan_name_text, .. }),.. } if fahrplan_name_text == betriebsstelle =>
                break,
            _ => {
                new.pop_front();
            }
        }
    }

    let (can_concat, laufweg_diff) = match (current.last(), new.front()) {
        (Some(previous_last), Some(next_first)) => (can_concat(previous_last, next_first), previous_last.fahrplan_laufweg - next_first.fahrplan_laufweg),
        _ => (false, 0.),
    };

    if can_concat {
        // current.pop() and new.front() are always Some if can_concat is true
        let current_last = current.pop().unwrap();
        if new.front().unwrap().fahrplan_ankunft.is_none() {
            new.front_mut().unwrap().fahrplan_ankunft = current_last.fahrplan_ankunft;
        }
        if new.front().unwrap().fahrplan_abfahrt.is_none() {
            new.front_mut().unwrap().fahrplan_abfahrt = current_last.fahrplan_abfahrt;
        }
        // TODO: check if FplAnk.FplEintrag needs to be adjusted
        new.iter_mut().for_each(|zeile| zeile.fahrplan_laufweg += laufweg_diff);
        current.append(&mut new.into());
        Ok(current)
    } else {
        Err(ConcatBuchfahrplaeneError::NonConsecutiveBuchfahrplaene)
    }
}

fn can_concat(first: &FahrplanZeile, second: &FahrplanZeile) -> bool {
    match (first, second) {
        (
            FahrplanZeile {
                fahrplan_km: previous_km,
                fahrplan_name: Some(FahrplanName { fahrplan_name_text: previous_betriebsstelle, .. }),
                fahrplan_abfahrt: previous_abfahrt,
                ..
            },
            FahrplanZeile {
                fahrplan_km: next_km,
                fahrplan_name: Some(FahrplanName { fahrplan_name_text: next_betriebsstelle, .. }),
                fahrplan_abfahrt: next_abfahrt,
                ..
            },
        ) if previous_km == next_km && previous_betriebsstelle == next_betriebsstelle =>
            previous_abfahrt.is_some() || next_abfahrt.is_some(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_icon::FahrplanIcon;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_km::FahrplanKm;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name_rechts::FahrplanNameRechts;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_signal_typ::FahrplanSignalTyp;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_v_max::FahrplanVMax;

    #[test]
    fn test_can_concat() {
        assert_eq!(
            can_concat(
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .build(),
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                    .build(),
            ),
            true,
        );
        assert_eq!(
            can_concat(
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                    .build(),
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .build(),
            ),
            true,
        );
        assert_eq!(
            can_concat(
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .build(),
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .build(),
            ),
            false,
        );
        assert_eq!(
            can_concat(
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                    .build(),
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("B".into()).build()))
                    .build(),
            ),
            false,
        );
        assert_eq!(
            can_concat(
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(39.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                    .build(),
                &FahrplanZeile::builder()
                    .fahrplan_km(vec![FahrplanKm::builder().km(2.7).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("A".into()).build()))
                    .build(),
            ),
            false,
        );
    }

    #[test]
    fn test_concat_buchfahrplaene() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(21799.445)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(1.7792).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(24631.027)
                .fahrplan_km(vec![FahrplanKm::builder().km(4.5357).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Mehle Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(3894.574)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.802).build()])
                .fahrplan_icon(vec![FahrplanIcon::builder().fahrplan_icon_nummer(17).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(6737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        let expected_buchfahrplan = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(21799.445)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(1.7792).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(24631.027)
                .fahrplan_km(vec![FahrplanKm::builder().km(4.5357).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Mehle Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(33894.574)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.802).build()])
                .fahrplan_icon(vec![FahrplanIcon::builder().fahrplan_icon_nummer(17).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(36737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap(),
            expected_buchfahrplan,
        );
    }

    #[test]
    fn test_concat_buchfahrplaene_without_last_abfahrt_and_first_ankunft() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(6737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        let expected_buchfahrplan = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(36737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap(),
            expected_buchfahrplan,
        );
    }

    #[test]
    fn test_concat_buchfahrplaene_without_last_last_ankunft_and_firt_abfahrt() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(6737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        let expected_buchfahrplan = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(36737.934)
                .fahrplan_km(vec![FahrplanKm::builder().km(16.6446).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Coppenbrügge Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:56:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:56:50)).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap(),
            expected_buchfahrplan,
        );
    }

    #[test]
    fn test_concat_buchfahrplaene_without_ankunft() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
        ];

        let expected_buchfahrplan = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap(),
            expected_buchfahrplan,
        );
    }

    #[test]
    fn test_cannot_concat_buchfahrplaene_with_unequal_text() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:52:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap_err(),
            ConcatBuchfahrplaeneError::NonConsecutiveBuchfahrplaene,
        );
    }

    #[test]
    fn test_cannot_concat_buchfahrplaene_without_abfahrt() {
        let buchfahrplan_1 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(32660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(32883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(33435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 50".into()).build()))
                .build(),
        ];

        let buchfahrplan_2 = vec![
            FahrplanZeile::builder()
                .fahrplan_laufweg(2660.822)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(22.2222).build()))
                .fahrplan_km(vec![FahrplanKm::builder().km(12.5721).build()])
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(2883.34)
                .fahrplan_km(vec![FahrplanKm::builder().km(12.7907).build()])
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Voldagsen".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:52:10)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_laufweg(3435.87)
                .fahrplan_km(vec![FahrplanKm::builder().km(13.3433).build()])
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(9).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("A 60".into()).build()))
                .build(),
        ];

        assert_eq!(
            concat_buchfahrplaene(buchfahrplan_1, buchfahrplan_2, "Voldagsen").unwrap_err(),
            ConcatBuchfahrplaeneError::NonConsecutiveBuchfahrplaene,
        );
    }
}