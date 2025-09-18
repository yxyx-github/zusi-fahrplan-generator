use serde_helpers::default::IsDefault;
use crate::core::lib::file_error::FileError;
use crate::core::lib::generated_zug::RawGeneratedZug;
use crate::core::lib::helpers::{override_non_default, override_with_non_default, read_buchfahrplan, read_zug};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::rolling_stock_config::RollingStockConfig;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ReplaceRollingStockError {
    #[error("Couldn't read the rolling stock template file: {error}")]
    ReadRollingStockError {
        #[source]
        error: FileError,
    },
    #[error("Couldn't read the 'Buchfahrplan' file for the given rolling stock template: {error}")]
    ReadBuchfahrplanError {
        #[source]
        error: FileError,
    },
}

pub fn replace_rolling_stock(env: &ZusiEnvironment, config: RollingStockConfig, zug: &mut RawGeneratedZug) -> Result<(), ReplaceRollingStockError> {
    let rolling_stock_template_path = env.path_to_prejoined_zusi_path(&config.path)
        .map_err(|error| ReplaceRollingStockError::ReadRollingStockError { error })?;
    let rolling_stock_template = read_zug(rolling_stock_template_path.full_path())
        .map_err(|error| ReplaceRollingStockError::ReadRollingStockError { error })?.value;

    zug.zug.fahrzeug_varianten = rolling_stock_template.fahrzeug_varianten;
    override_with_non_default(&mut zug.zug.fahrplan_zug_laenge, rolling_stock_template.fahrplan_zug_laenge);
    override_with_non_default(&mut zug.zug.baureihe_angabe, rolling_stock_template.baureihe_angabe);
    override_with_non_default(&mut zug.zug.bremsstellung_zug, rolling_stock_template.bremsstellung_zug);
    override_with_non_default(&mut zug.zug.fahrplan_bremsstellung_textvorgabe, rolling_stock_template.fahrplan_bremsstellung_textvorgabe);
    override_with_non_default(&mut zug.zug.fahrplan_masse, rolling_stock_template.fahrplan_masse);
    override_with_non_default(&mut zug.zug.grenzlast, rolling_stock_template.grenzlast);
    override_with_non_default(&mut zug.zug.speed_zug_niedriger, rolling_stock_template.speed_zug_niedriger);
    override_with_non_default(&mut zug.zug.tuer_system_bezeichner, rolling_stock_template.tuer_system_bezeichner);

    if let (
        Some(buchfahrplan),
        Some(Datei { dateiname, .. }),
    ) = (
        &mut zug.buchfahrplan,
        rolling_stock_template.buchfahrplan_roh_datei,
    ) {
        let buchfahrplan_path = env.zusi_path_to_prejoined_zusi_path(dateiname);
        let rolling_stock_buchfahrplan = read_buchfahrplan(buchfahrplan_path.full_path())
            .map_err(|error| ReplaceRollingStockError::ReadBuchfahrplanError { error })?.value;

        override_with_non_default(&mut buchfahrplan.bremshundertstel, rolling_stock_buchfahrplan.bremshundertstel);
        override_with_non_default(&mut zug.zug.mindest_bremshundertstel, buchfahrplan.mindest_bremshundertstel);
        override_with_non_default(&mut buchfahrplan.laenge, rolling_stock_buchfahrplan.laenge);
        override_with_non_default(&mut zug.zug.fahrplan_zug_laenge, buchfahrplan.laenge);
        override_with_non_default(&mut buchfahrplan.laenge_loks, rolling_stock_buchfahrplan.laenge_loks);
        override_with_non_default(&mut buchfahrplan.wagenzug_laenge, rolling_stock_buchfahrplan.wagenzug_laenge);
        override_with_non_default(&mut buchfahrplan.fahrzeug_info, rolling_stock_buchfahrplan.fahrzeug_info);
        override_with_non_default(&mut buchfahrplan.baureihe, rolling_stock_buchfahrplan.baureihe);
        override_with_non_default(&mut zug.zug.baureihe_angabe, buchfahrplan.baureihe.clone());
        override_with_non_default(&mut buchfahrplan.bremsstellung_zug, rolling_stock_buchfahrplan.bremsstellung_zug);
        override_with_non_default(&mut zug.zug.bremsstellung_zug, buchfahrplan.bremsstellung_zug.clone());
        override_with_non_default(&mut buchfahrplan.fahrplan_bremsstellung_textvorgabe, rolling_stock_buchfahrplan.fahrplan_bremsstellung_textvorgabe);
        override_with_non_default(&mut zug.zug.fahrplan_bremsstellung_textvorgabe, buchfahrplan.fahrplan_bremsstellung_textvorgabe.clone());
        override_with_non_default(&mut buchfahrplan.masse, rolling_stock_buchfahrplan.masse);
        override_with_non_default(&mut zug.zug.fahrplan_masse, buchfahrplan.masse);
        override_with_non_default(&mut buchfahrplan.grenzlast, rolling_stock_buchfahrplan.grenzlast);
        override_with_non_default(&mut zug.zug.grenzlast, buchfahrplan.grenzlast);
        let max_speed = if zug.zug.speed_zug_niedriger.is_default() || rolling_stock_buchfahrplan.speed_max < zug.zug.speed_zug_niedriger {
            rolling_stock_buchfahrplan.speed_max
        } else {
            zug.zug.speed_zug_niedriger
        };
        override_with_non_default(&mut buchfahrplan.speed_max, max_speed);
        override_non_default(&mut zug.zug.speed_zug_niedriger, buchfahrplan.speed_max);
    } else if zug.buchfahrplan.is_some() {
        zug.buchfahrplan = None;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lib::helpers::empty_buchfahrplan;
    use std::fs;
    use tempfile::tempdir;
    use zusi_xml_lib::xml::zusi::buchfahrplan::Buchfahrplan;
    use zusi_xml_lib::xml::zusi::lib::bremsstellung::Bremsstellung;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::lib::utm::UTM;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::fahrzeug_info::{DoppeltraktionsModus, FahrzeugInfo};
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
    use zusi_xml_lib::xml::zusi::zug::Zug;

    const ROLLING_STOCK_TEMPLATE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F" MBrh="1.7" FplMasse="300" FplZuglaenge="100" TuerSystemBezeichner="TAV" BRAngabe="ET 1234" Grenzlast="1" spZugNiedriger="20">
                <Datei/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="2" DotraModus="1">
                        <Datei Dateiname="path/to/A-Wagen.fzg"/>
                    </FahrzeugInfo>
                    <FahrzeugInfo IDHaupt="2" IDNeben="2" DotraModus="1" Gedreht="1">
                        <Datei Dateiname="path/to/B-Wagen.fzg"/>
                    </FahrzeugInfo>
                    <FahrzeugInfo IDHaupt="1" IDNeben="2" DotraModus="1">
                        <Datei Dateiname="path/to/C-Wagen.fzg"/>
                    </FahrzeugInfo>
                    <FahrzeugInfo IDHaupt="2" IDNeben="2" DotraModus="1" Gedreht="1">
                        <Datei Dateiname="path/to/D-Wagen.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TEMPLATE_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F" MBrh="1.7" FplMasse="300" FplZuglaenge="90" TuerSystemBezeichner="TAV" BRAngabe="ET 4321" Grenzlast="1" spZugNiedriger="20" BremsstellungZug="5">
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="00000.timetable.xml"/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="2" DotraModus="1">
                        <Datei Dateiname="path/to/A-Wagen.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0"/>
            <Buchfahrplan Gattung="RB" Nummer="00000" BR="ET 4321" Laenge="100" Masse="350" Grenzlast="1" spMax="25" MBrh="1.4" BremsstellungZug="3">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM/>
            </Buchfahrplan>
        </Zusi>
    "#;

    #[test]
    fn test_replace_rolling_stock() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00000.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let mut zug = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .bremsstellung_zug(Bremsstellung::RMg)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/B-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(2)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .gedreht(true)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = RollingStockConfig {
            path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .fahrplan_masse(300.)
                .fahrplan_zug_laenge(100.0)
                .tuer_system_bezeichner("TAV".into())
                .bremsstellung_zug(Bremsstellung::RMg)
                .baureihe_angabe("ET 1234".into())
                .grenzlast(true)
                .speed_zug_niedriger(20.0)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/B-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(2)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .gedreht(true)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/C-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/D-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(2)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .gedreht(true)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: None,
        };

        replace_rolling_stock(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE);
    }

    #[test]
    fn test_replace_rolling_stock_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00000.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE_WITH_TIMETABLE).unwrap();

        let timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&timetable_path, TIMETABLE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let mut zug = RawGeneratedZug {
            zug: Zug::builder()
                .nummer("00000".into())
                .gattung("RB".into())
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .bremsstellung_zug(Bremsstellung::R)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/B-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(2)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .gedreht(true)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: Some(empty_buchfahrplan()),
        };

        let config = RollingStockConfig {
            path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = RawGeneratedZug {
            zug: Zug::builder()
                .nummer("00000".into())
                .gattung("RB".into())
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .fahrplan_masse(350.)
                .fahrplan_zug_laenge(100.0)
                .tuer_system_bezeichner("TAV".into())
                .bremsstellung_zug(Bremsstellung::PMg)
                .baureihe_angabe("ET 4321".into())
                .grenzlast(true)
                .speed_zug_niedriger(20.0)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: Some(Buchfahrplan::builder()
                .baureihe("ET 4321".into())
                .mindest_bremshundertstel(0.)
                .masse(350.)
                .speed_max(20.)
                .grenzlast(true)
                .laenge(100.)
                .bremsstellung_zug(Bremsstellung::PMg)
                .datei_fpn(Datei::builder().build())
                .datei_trn(Datei::builder().build())
                .utm(UTM::builder().build())
                .build()),
        };

        replace_rolling_stock(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(timetable_path).unwrap(), TIMETABLE);
    }

    #[test]
    fn test_replace_rolling_stock_with_ignored_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00000.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE_WITH_TIMETABLE).unwrap();

        let timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&timetable_path, TIMETABLE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let mut zug = RawGeneratedZug {
            zug: Zug::builder()
                .nummer("00000".into())
                .gattung("RB".into())
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .bremsstellung_zug(Bremsstellung::R)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/B-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(2)
                                .id_neben(1)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .gedreht(true)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = RollingStockConfig {
            path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = RawGeneratedZug {
            zug: Zug::builder()
                .nummer("00000".into())
                .gattung("RB".into())
                .fahrplan_datei(Datei::builder().build())
                .mindest_bremshundertstel(1.9)
                .fahrplan_masse(300.)
                .fahrplan_zug_laenge(90.0)
                .tuer_system_bezeichner("TAV".into())
                .bremsstellung_zug(Bremsstellung::RMg)
                .baureihe_angabe("ET 4321".into())
                .grenzlast(true)
                .speed_zug_niedriger(20.0)
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("path/to/A-Wagen.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(2)
                                .doppeltraktions_modus(DoppeltraktionsModus::Mehrfachtraktion)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: None,
        };

        replace_rolling_stock(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(timetable_path).unwrap(), TIMETABLE);
    }
}