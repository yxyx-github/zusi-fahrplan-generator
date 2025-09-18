use crate::core::lib::generated_zug::RawGeneratedZug;
use crate::core::lib::helpers::{delay_fahrplan_eintraege, delay_fahrplan_zeilen};
use crate::core::lib::zug_nummer::ZugNummer;
use crate::core::replace_rolling_stock::{replace_rolling_stock, ReplaceRollingStockError};
use crate::input::copy_delay_config::{CopyDelayConfig, CopyDelayTask};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CopyDelayError {
    #[error("The rolling stock couldn't be replaced: {error}")]
    ReplaceRollingStockError {
        #[from]
        error: ReplaceRollingStockError,
    },

    #[error("The 'Zugnummer' is invalid: {error}")]
    InvalidZugNummer {
        #[from]
        error: ParseIntError,
    },

    #[error("The 'Zugnummer' always must be positive. It can be decremented only if the resulting value is still positive.")]
    ZugNummerCanNotBeNegative,
}

pub fn copy_delay(env: &ZusiEnvironment, config: CopyDelayConfig, zug: &RawGeneratedZug) -> Result<Vec<RawGeneratedZug>, CopyDelayError> {
    config.tasks.into_iter().try_fold(
        vec![],
        |mut zuege, task| {
            zuege.append(&mut apply_copy_delay_task(env, task, zug)?);
            Ok(zuege)
        },
    )
}

fn apply_copy_delay_task(env: &ZusiEnvironment, task: CopyDelayTask, zug: &RawGeneratedZug) -> Result<Vec<RawGeneratedZug>, CopyDelayError> {
    let mut zug = zug.clone();
    let zug_nummer = ZugNummer::try_from(&zug.zug.nummer)?;
    let zug = match task.custom_rolling_stock {
        None => zug,
        Some(replace_rolling_stock_config) => {
            replace_rolling_stock(env, replace_rolling_stock_config, &mut zug)?;
            zug
        }
    };
    (1..=task.count).into_iter().try_fold(
        vec![],
        |mut zuege, n| {
            let mut zug = zug.clone();
            zug.zug.nummer = zug_nummer
                .to_new_incremented(n as i32 * task.increment)
                .map_err(|_| CopyDelayError::ZugNummerCanNotBeNegative)?
                .into();

            delay_fahrplan_eintraege(&mut zug.zug.fahrplan_eintraege, n * task.delay);
            if let Some(ref mut buchfahrplan) = zug.buchfahrplan {
                delay_fahrplan_zeilen(&mut buchfahrplan.fahrplan_zeilen, n * task.delay);
                buchfahrplan.nummer = zug.zug.nummer.clone();
            }

            zuege.push(zug);
            Ok(zuege)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::rolling_stock_config::RollingStockConfig;
    use std::fs;
    use tempfile::tempdir;
    use time::macros::datetime;
    use time::Duration;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
    use zusi_xml_lib::xml::zusi::buchfahrplan::Buchfahrplan;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::lib::utm::UTM;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::fahrzeug_info::FahrzeugInfo;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
    use zusi_xml_lib::xml::zusi::zug::Zug;

    const ROLLING_STOCK_TEMPLATE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug>
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="00000.timetable.xml"/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TIMETABLE: &str = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <Zusi>
                <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
                <Buchfahrplan Gattung="RB" Nummer="00000">
                    <Datei_fpn/>
                    <Datei_trn/>
                    <UTM/>
                </Buchfahrplan>
            </Zusi>
        "#;

    #[test]
    fn test_copy_delay() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00001.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE).unwrap();

        let rolling_stock_timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&rolling_stock_timetable_path, ROLLING_STOCK_TIMETABLE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .nummer("342_702".into())
                .fahrplan_eintraege(vec![
                    FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 02:20:50))).abfahrt(Some(datetime!(2004-07-09 02:21:30))).build(),
                    FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 02:23:30))).build(),
                ])
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("TriebwagenB.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(1)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(4),
                    count: 2,
                    increment: 7,
                    custom_rolling_stock: Some(RollingStockConfig { path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
                },
                CopyDelayTask {
                    delay: Duration::hours(-1),
                    count: 2,
                    increment: -2,
                    custom_rolling_stock: None,
                },
            ],
        };

        let expected = vec![
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("349_709".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 06:20:50))).abfahrt(Some(datetime!(2004-07-09 06:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 06:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenA.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: None,
            },
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("356_716".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 10:20:50))).abfahrt(Some(datetime!(2004-07-09 10:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 10:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenA.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: None,
            },
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("340_700".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 01:20:50))).abfahrt(Some(datetime!(2004-07-09 01:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 01:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenB.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: None,
            },
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("338_698".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 00:20:50))).abfahrt(Some(datetime!(2004-07-09 00:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 00:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenB.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: None,
            },
        ];

        assert_eq!(
            copy_delay(&env, config, &input).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE);
        assert_eq!(fs::read_to_string(rolling_stock_timetable_path).unwrap(), ROLLING_STOCK_TIMETABLE);
    }

    #[test]
    fn test_copy_delay_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let rolling_stock_template_path = tmp_dir.path().join("00001.trn");
        fs::write(&rolling_stock_template_path, ROLLING_STOCK_TEMPLATE).unwrap();

        let rolling_stock_timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&rolling_stock_timetable_path, ROLLING_STOCK_TIMETABLE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .nummer("342_702".into())
                .fahrplan_eintraege(vec![
                    FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 02:20:50))).abfahrt(Some(datetime!(2004-07-09 02:21:30))).build(),
                    FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 02:23:30))).build(),
                ])
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder()
                        .bezeichnung("default".into())
                        .zufalls_wert(1.)
                        .fahrzeug_infos(vec![
                            FahrzeugInfo::builder()
                                .datei(Datei::builder().dateiname("TriebwagenB.fzg".try_into().unwrap()).build())
                                .id_haupt(1)
                                .id_neben(1)
                                .build(),
                        ])
                        .build()
                )
                .build(),
            buchfahrplan: Some(
                Buchfahrplan::builder()
                    .nummer("342_702".into())
                    .fahrplan_zeilen(vec![
                        FahrplanZeile::builder()
                            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2004-07-09 02:20:50)).build()))
                            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 02:21:30)).build()))
                            .build(),
                        FahrplanZeile::builder()
                            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 02:23:30)).build()))
                            .build(),
                    ])
                    .datei_fpn(Datei::builder().build())
                    .datei_trn(Datei::builder().build())
                    .utm(UTM::builder().build())
                    .build()
            ),
        };

        let config = CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(4),
                    count: 1,
                    increment: 7,
                    custom_rolling_stock: Some(RollingStockConfig { path: rolling_stock_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
                },
                CopyDelayTask {
                    delay: Duration::hours(-1),
                    count: 1,
                    increment: -2,
                    custom_rolling_stock: None,
                },
            ],
        };

        let expected = vec![
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("349_709".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 06:20:50))).abfahrt(Some(datetime!(2004-07-09 06:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 06:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenA.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: Some(
                    Buchfahrplan::builder()
                        .nummer("349_709".into())
                        .fahrplan_zeilen(vec![
                            FahrplanZeile::builder()
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2004-07-09 06:20:50)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 06:21:30)).build()))
                                .build(),
                            FahrplanZeile::builder()
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 06:23:30)).build()))
                                .build(),
                        ])
                        .datei_fpn(Datei::builder().build())
                        .datei_trn(Datei::builder().build())
                        .utm(UTM::builder().build())
                        .build()
                ),
            },
            RawGeneratedZug {
                zug: Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .nummer("340_700".into())
                    .fahrplan_eintraege(vec![
                        FahrplanEintrag::builder().betriebsstelle("ADorf".into()).ankunft(Some(datetime!(2004-07-09 01:20:50))).abfahrt(Some(datetime!(2004-07-09 01:21:30))).build(),
                        FahrplanEintrag::builder().betriebsstelle("BDorf".into()).abfahrt(Some(datetime!(2004-07-09 01:23:30))).build(),
                    ])
                    .fahrzeug_varianten(
                        FahrzeugVarianten::builder()
                            .bezeichnung("default".into())
                            .zufalls_wert(1.)
                            .fahrzeug_infos(vec![
                                FahrzeugInfo::builder()
                                    .datei(Datei::builder().dateiname("TriebwagenB.fzg".try_into().unwrap()).build())
                                    .id_haupt(1)
                                    .id_neben(1)
                                    .build(),
                            ])
                            .build()
                    )
                    .build(),
                buchfahrplan: Some(
                    Buchfahrplan::builder()
                        .nummer("340_700".into())
                        .fahrplan_zeilen(vec![
                            FahrplanZeile::builder()
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("ADorf".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2004-07-09 01:20:50)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 01:21:30)).build()))
                                .build(),
                            FahrplanZeile::builder()
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("BDorf".into()).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2004-07-09 01:23:30)).build()))
                                .build(),
                        ])
                        .datei_fpn(Datei::builder().build())
                        .datei_trn(Datei::builder().build())
                        .utm(UTM::builder().build())
                        .build()
                ),
            },
        ];

        assert_eq!(
            copy_delay(&env, config, &input).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(rolling_stock_template_path).unwrap(), ROLLING_STOCK_TEMPLATE);
        assert_eq!(fs::read_to_string(rolling_stock_timetable_path).unwrap(), ROLLING_STOCK_TIMETABLE);
    }

    #[test]
    fn test_copy_delay_with_rolling_stock_error() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .nummer("342_702".into())
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder().build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(4),
                    count: 2,
                    increment: 7,
                    custom_rolling_stock: Some(RollingStockConfig { path: "non-existent".into() }),
                },
            ],
        };

        assert!(matches!(
            copy_delay(&env, config, &input).unwrap_err(),
            CopyDelayError::ReplaceRollingStockError { .. },
        ));
    }

    #[test]
    fn test_copy_delay_invalid_zug_nummer() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .nummer("invalid".into())
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder().build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(-1),
                    count: 2,
                    increment: -2,
                    custom_rolling_stock: None,
                },
            ],
        };

        assert!(matches!(
            copy_delay(&env, config, &input).unwrap_err(),
            CopyDelayError::InvalidZugNummer { .. },
        ));
    }

    #[test]
    fn test_copy_delay_zug_nummer_must_not_be_negative() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let input = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .nummer("2".into())
                .fahrzeug_varianten(
                    FahrzeugVarianten::builder().build()
                )
                .build(),
            buchfahrplan: None,
        };

        let config = CopyDelayConfig {
            tasks: vec![
                CopyDelayTask {
                    delay: Duration::hours(-1),
                    count: 2,
                    increment: -3,
                    custom_rolling_stock: None,
                },
            ],
        };

        assert!(matches!(
            copy_delay(&env, config, &input).unwrap_err(),
            CopyDelayError::ZugNummerCanNotBeNegative { .. },
        ));
    }
}