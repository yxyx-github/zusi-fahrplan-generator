use thiserror::Error;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UpdateBuchfahrplanError {
    #[error("The number of entries for 'FahrplanZeile' and 'FahrplanEintrag' must be equal.")]
    InvalidLen,

    #[error("Related entries of type 'FahrplanZeile' and 'FahrplanEintrag' must fulfill following criteria: 'Betriebsstelle' is equal, either 'Ankunft' or 'Abfahrt' must be set.")]
    RelatedEntriesMustBeEqual, // TODO: specify failed criteria
}

pub fn update_buchfahrplan(fahrplan_eintraege: &Vec<FahrplanEintrag>, fahrplan_zeilen: &mut Vec<FahrplanZeile>) -> Result<(), UpdateBuchfahrplanError> {
    let fahrplan_eintraege: Vec<&FahrplanEintrag> = fahrplan_eintraege
        .iter()
        .filter(|eintrag| eintrag.abfahrt.is_some())
        .collect();

    let fahrplan_zeilen: Vec<&mut FahrplanZeile> = fahrplan_zeilen
        .iter_mut()
        .filter(|zeile| zeile.fahrplan_ankunft.is_some() || zeile.fahrplan_abfahrt.is_some())
        .map(|zeile| zeile)
        .collect();

    if fahrplan_zeilen.len() != fahrplan_eintraege.len() {
        return Err(UpdateBuchfahrplanError::InvalidLen);
    }

    let zipped_entries: Vec<_> = fahrplan_eintraege.into_iter().zip(fahrplan_zeilen.into_iter()).collect();

    zipped_entries
        .into_iter()
        .try_for_each(|(fahrplan_eintrag, fahrplan_zeile)| {
            match fahrplan_zeile.fahrplan_name {
                Some(FahrplanName { ref fahrplan_name_text, .. }) if fahrplan_eintrag.betriebsstelle == *fahrplan_name_text => {}
                _ => return Err(UpdateBuchfahrplanError::RelatedEntriesMustBeEqual),
            }
            match fahrplan_zeile.fahrplan_ankunft {
                None if fahrplan_zeile.fahrplan_abfahrt.is_some() => {}
                Some(FahrplanAnkunft { ref mut ankunft, .. }) if fahrplan_eintrag.ankunft.is_some() => *ankunft = fahrplan_eintrag.ankunft.unwrap(),
                _ => return Err(UpdateBuchfahrplanError::RelatedEntriesMustBeEqual),
            }
            match fahrplan_zeile.fahrplan_abfahrt {
                None if fahrplan_zeile.fahrplan_ankunft.is_some() => {}
                Some(FahrplanAbfahrt { ref mut abfahrt, .. }) if fahrplan_eintrag.abfahrt.is_some() => *abfahrt = fahrplan_eintrag.abfahrt.unwrap(),
                _ => return Err(UpdateBuchfahrplanError::RelatedEntriesMustBeEqual), // unreachable because entries are already filtered
            }
            Ok(())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_km::FahrplanKm;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name_rechts::FahrplanNameRechts;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_signal_typ::FahrplanSignalTyp;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_v_max::FahrplanVMax;

    #[test]
    fn test_update_buchfahrplan() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .abfahrt(Some(datetime!(2024-06-20 08:46:00)))
                .betriebsstelle("Mehle Hp".into())
                .build(),
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(21799.445)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(Some(FahrplanKm::builder().km(1.7792).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(24631.027)
                .fahrplan_km(Some(FahrplanKm::builder().km(4.5357).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Mehle Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:06:00)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(Some(FahrplanKm::builder().km(12.128).build()))
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
        ];

        let expected_fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(21799.445)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(Some(FahrplanKm::builder().km(1.7792).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(24631.027)
                .fahrplan_km(Some(FahrplanKm::builder().km(4.5357).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Mehle Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:46:00)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(Some(FahrplanKm::builder().km(12.128).build()))
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
        ];

        update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap();

        assert_eq!(fahrplan_zeilen, expected_fahrplan_zeilen);
    }

    #[test]
    fn test_update_buchfahrplan_with_ankunft_only() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .build(),
        ];

        let expected_fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:49:00)).build()))
                .build(),
        ];

        update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap();

        assert_eq!(fahrplan_zeilen, expected_fahrplan_zeilen);
    }

    #[test]
    fn test_update_buchfahrplan_with_abfahrt_only() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:39:00)).build()))
                .build(),
        ];

        let expected_fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:49:50)).build()))
                .build(),
        ];

        update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap();

        assert_eq!(fahrplan_zeilen, expected_fahrplan_zeilen);
    }

    #[test]
    fn test_update_buchfahrplan_with_invalid_len() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .abfahrt(Some(datetime!(2024-06-20 08:46:00)))
                .betriebsstelle("Mehle Hp".into())
                .build(),
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(21799.445)
                .fahrplan_v_max(Some(FahrplanVMax::builder().v_max(33.3333).build()))
                .fahrplan_km(Some(FahrplanKm::builder().km(1.7792).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:49:50)).build()))
                .build(),
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(32220.396)
                .fahrplan_km(Some(FahrplanKm::builder().km(12.128).build()))
                .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                .build(),
        ];

        assert_eq!(
            update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap_err(),
            UpdateBuchfahrplanError::InvalidLen,
        );
    }

    #[test]
    fn test_update_buchfahrplan_with_unequal_betriebsstelle() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hst".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:49:50)).build()))
                .build(),
        ];

        assert_eq!(
            update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap_err(),
            UpdateBuchfahrplanError::RelatedEntriesMustBeEqual,
        );
    }

    #[test]
    fn test_update_buchfahrplan_with_unequal_ankunft() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:49:50)).build()))
                .build(),
        ];

        assert_eq!(
            update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap_err(),
            UpdateBuchfahrplanError::RelatedEntriesMustBeEqual,
        );
    }

    #[test]
    fn test_update_buchfahrplan_with_missing_ankunft_and_abfahrt() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .ankunft(Some(datetime!(2024-06-20 08:49:00)))
                .abfahrt(Some(datetime!(2024-06-20 08:49:50)))
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .build(),
        ];

        assert_eq!(
            update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap_err(),
            UpdateBuchfahrplanError::InvalidLen,
        );
    }

    #[test]
    fn test_update_buchfahrplan_with_unequal_abfahrt() {
        let fahrplan_eintraege = vec![
            FahrplanEintrag::builder()
                .signal_vorlauf(160.)
                .betriebsstelle("Osterwald Hp".into())
                .build(),
        ];

        let mut fahrplan_zeilen = vec![
            FahrplanZeile::builder()
                .fahrplan_regelgleis_gegengleis(1)
                .fahrplan_laufweg(29134.139)
                .fahrplan_km(Some(FahrplanKm::builder().km(9.0405).build()))
                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:49:50)).build()))
                .build(),
        ];

        assert_eq!(
            update_buchfahrplan(&fahrplan_eintraege, &mut fahrplan_zeilen).unwrap_err(),
            UpdateBuchfahrplanError::InvalidLen,
        );
    }
}