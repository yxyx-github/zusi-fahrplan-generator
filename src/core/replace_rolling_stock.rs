use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::read_zug;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::rolling_stock_config::RollingStockConfig;
use serde_helpers::default::IsDefault;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::zug::Zug;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ReplaceRollingStockError {
    #[error("Couldn't read the rolling stock template file: {error}")]
    ReadRollingStockError {
        error: FileError,
    },
}

pub fn replace_rolling_stock(env: &ZusiEnvironment, config: RollingStockConfig, mut zug: Zug) -> Result<Zug, ReplaceRollingStockError> {
    let rolling_stock_template_path = env.path_to_prejoined_zusi_path(&config.path)
        .map_err(|error| ReplaceRollingStockError::ReadRollingStockError { error: (&config.path, error).into() })?;
    let rolling_stock_template = read_zug(rolling_stock_template_path.full_path())
        .map_err(|error| ReplaceRollingStockError::ReadRollingStockError { error })?;

    zug.fahrzeug_varianten = rolling_stock_template.value.fahrzeug_varianten;

    override_unset(&mut zug.mindest_bremshundertstel, rolling_stock_template.value.mindest_bremshundertstel);
    override_unset(&mut zug.fahrplan_zug_laenge, rolling_stock_template.value.fahrplan_zug_laenge);
    override_unset(&mut zug.fahrplan_masse, rolling_stock_template.value.fahrplan_masse);
    override_unset(&mut zug.bremsstellung_zug, rolling_stock_template.value.bremsstellung_zug);
    override_unset(&mut zug.fahrplan_bremsstellung_textvorgabe, rolling_stock_template.value.fahrplan_bremsstellung_textvorgabe);
    override_unset(&mut zug.tuer_system_bezeichner, rolling_stock_template.value.tuer_system_bezeichner);

    Ok(zug)
}

fn override_unset<T: IsDefault>(a: &mut T, b: T) {
    if a.is_default() {
        *a = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use zusi_xml_lib::xml::zusi::lib::bremsstellung::Bremsstellung;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::fahrzeug_info::{DoppeltraktionsModus, FahrzeugInfo};
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;

    const ROLLING_STOCK_TEMPLATE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F" MBrh="1.7" FplMasse="300" FplZuglaenge="100" TuerSystemBezeichner="TAV">
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

    #[test]
    fn test_replace_rolling_stock() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00001.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = Zug::builder()
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
            .build();

        let config = RollingStockConfig {
            path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = Zug::builder()
            .fahrplan_datei(Datei::builder().build())
            .mindest_bremshundertstel(1.9)
            .fahrplan_masse(300.)
            .fahrplan_zug_laenge(100.0)
            .tuer_system_bezeichner("TAV".into())
            .bremsstellung_zug(Bremsstellung::RMg)
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
            .build();

        assert_eq!(replace_rolling_stock(&env, config, input).unwrap(), expected);

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE);
    }
}