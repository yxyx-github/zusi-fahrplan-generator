mod generate_route;
mod add_meta_data;

use crate::core::generate_fahrplan::generate_zug::add_meta_data::{add_meta_data, AddMetaDataError};
use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::apply_resolved_route_to_zug;
use crate::core::generate_fahrplan::generate_zug::generate_route::{generate_route, GenerateRouteError};
use crate::core::lib::copy_delay::{copy_delay, CopyDelayError};
use crate::core::lib::file_error::FileError;
use crate::core::lib::generated_zug::{GeneratedZug, RawGeneratedZug};
use crate::core::lib::helpers::{datei_from_prejoined_zusi_path, empty_buchfahrplan_with_gattung_and_nummer, override_with_non_default};
use crate::core::replace_rolling_stock::{replace_rolling_stock, ReplaceRollingStockError};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::ZugConfig;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
use zusi_xml_lib::xml::zusi::zug::Zug;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("Couldn't generate 'Zug' with 'Zugnummer' {zug_nummer}: {error}")]
pub struct GenerateZugError {
    zug_nummer: String,
    error: GenerateZugErrorKind,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GenerateZugErrorKind {
    #[error("The route couldn't be generated: {error}")]
    GenerateRouteError {
        #[from]
        error: GenerateRouteError,
    },

    #[error("The 'Fahrplan' file couldn't be attached: {error}")]
    AttachFahrplanFileError {
        error: FileError,
    },

    #[error("The meta data couldn't be added: {error}")]
    AddMetaDataError {
        #[from]
        error: AddMetaDataError,
    },

    #[error("The rolling stock couldn't be replaced: {error}")]
    ReplaceRollingStockError {
        #[from]
        error: ReplaceRollingStockError,
    },

    #[error("Couldn't copy delay the 'Zug': {error}")]
    CopyDelayError {
        #[from]
        error: CopyDelayError,
    },
}

impl From<(&String, GenerateZugErrorKind)> for GenerateZugError {
    fn from((zug_nummer, error): (&String, GenerateZugErrorKind)) -> Self {
        Self {
            zug_nummer: zug_nummer.into(),
            error,
        }
    }
}

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, zug_config: ZugConfig) -> Result<Vec<GeneratedZug>, GenerateZugError> {
    let fahrplan_datei = datei_from_prejoined_zusi_path(fahrplan_path, true)
        .map_err(|error| GenerateZugError::from((&zug_config.nummer, GenerateZugErrorKind::AttachFahrplanFileError { error })))?;

    let mut zug = Zug::builder()
        .gattung(zug_config.gattung)
        .nummer(zug_config.nummer)
        .fahrplan_datei(fahrplan_datei)
        .fahrzeug_varianten(FahrzeugVarianten::builder().build())
        .build();

    let route = generate_route(env, zug_config.route)
        .map_err(|error| GenerateZugError::from((&zug.nummer, error.into())))?;

    override_with_non_default(&mut zug.mindest_bremshundertstel, route.mindest_bremshundertstel);

    let buchfahrplan = if route.fahrplan_zeilen.is_empty() {
        None
    } else {
        let mut buchfahrplan = empty_buchfahrplan_with_gattung_and_nummer(zug.gattung.clone(), zug.nummer.clone());
        if let Some(km_start) = route.start_data.km_start {
            buchfahrplan.km_start = km_start;
        }
        if let Some(gnt_spalte) = route.start_data.gnt_spalte {
            buchfahrplan.gnt_spalte = gnt_spalte;
        }
        override_with_non_default(&mut buchfahrplan.mindest_bremshundertstel, route.mindest_bremshundertstel);
        Some(buchfahrplan)
    };

    let mut zug: RawGeneratedZug = (zug, buchfahrplan).into();

    apply_resolved_route_to_zug(route, &mut zug);

    replace_rolling_stock(env, zug_config.rolling_stock, &mut zug)
        .map_err(|error| GenerateZugError::from((&zug.zug.nummer, error.into())))?;

    if let Some(meta_data) = zug_config.meta_data {
        add_meta_data(env, meta_data, &mut zug)
            .map_err(|error| GenerateZugError::from((&zug.zug.nummer, error.into())))?;
    }

    let mut zuege = vec![zug];

    if let Some(copy_delay_config) = zug_config.copy_delay_config {
        let raw_generated_zug = zuege.first().unwrap();
        let mut additional = copy_delay(env, copy_delay_config, raw_generated_zug)
            .map_err(|error| GenerateZugError::from((&raw_generated_zug.zug.nummer, error.into())))?;
        zuege.append(&mut additional);
    }

    Ok(zuege.into_iter().map(|raw| raw.into()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::copy_delay_config::{CopyDelayConfig, CopyDelayTask};
    use crate::input::fahrplan_config::{MetaDataConfig, RouteConfig, RoutePart, RoutePartSource};
    use crate::input::rolling_stock_config::RollingStockConfig;
    use std::fs;
    use tempfile::tempdir;
    use time::macros::datetime;
    use time::Duration;
    use zusi_xml_lib::xml::zusi::buchfahrplan::Buchfahrplan;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_km::FahrplanKm;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_v_max::FahrplanVMax;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
    use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
    use zusi_xml_lib::xml::zusi::lib::bremsstellung::Bremsstellung;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPath;
    use zusi_xml_lib::xml::zusi::lib::utm::UTM;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::fahrzeug_info::FahrzeugInfo;
    use zusi_xml_lib::xml::zusi::TypedZusi;

    const ROUTE_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROUTE_TRN_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="test/10001.timetable.xml"/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROUTE_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
            <Buchfahrplan Gattung="RB" Nummer="00002" MBrh="1.4">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
                <FplZeile FplLaufweg="20092.018">
                    <Fplkm km="32.8757" />
                    <FplName FplNameText="Elze" />
                    <FplAnk Ank="2024-06-20 08:39:00" />
                    <FplAbf Abf="2024-06-20 08:41:40" />
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="21799.445">
                    <FplvMax vMax="33.3333"/>
                    <Fplkm km="1.7792"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="24631.027">
                    <Fplkm km="4.5357"/>
                    <FplName FplNameText="Mehle Hp"/>
                    <FplAbf Abf="2024-06-20 08:45:00"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                    <Fplkm km="9.0405"/>
                    <FplName FplNameText="Osterwald Hp"/>
                    <FplAnk Ank="2024-06-20 08:48:00"/>
                    <FplAbf Abf="2024-06-20 08:48:40"/>
                </FplZeile>
            </Buchfahrplan>
        </Zusi>
    "#;

    const META_DATA_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug Zuglauf="ADorf - BDorf">
                <Datei/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug>
                <Datei/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TRN_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug>
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="test/dev/rolling-stock/Triebwagen-A.timetable.xml"/>
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
            <Buchfahrplan Gattung="RE" Nummer="99999" spMax="20.0" MBrh="1.4" BremsstellungZug="3">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
            </Buchfahrplan>
        </Zusi>
    "#;

    #[test]
    fn test_generate_zug() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let fpn_path = tmp_dir.path().join("test.fpn");
        let prejoined_fpn_path = PrejoinedZusiPath::new(&env.data_dir, ZusiPath::new_using_data_dir(fpn_path, &env.data_dir).unwrap());

        let route_path = tmp_dir.path().join("test/10001.trn");
        fs::create_dir_all(route_path.parent().unwrap()).unwrap();
        fs::write(&route_path, ROUTE_TRN).unwrap();

        let meta_data_path = tmp_dir.path().join("test/dev/meta-data.trn");
        fs::create_dir_all(meta_data_path.parent().unwrap()).unwrap();
        fs::write(&meta_data_path, META_DATA_TRN).unwrap();

        let rolling_stock_path = tmp_dir.path().join("test/dev/rolling-stock/Triebwagen-A.trn");
        fs::create_dir_all(rolling_stock_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_path, ROLLING_STOCK_TRN).unwrap();

        let config = ZugConfig {
            nummer: "10001".into(),
            gattung: "RB".into(),
            meta_data: Some(MetaDataConfig {
                path: meta_data_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            }),
            route: RouteConfig {
                parts: vec![
                    RoutePart {
                        source: RoutePartSource::TrainFileByPath { path: route_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                        time_fix: None,
                        apply_schedule: None,
                    },
                ],
            },
            rolling_stock: RollingStockConfig {
                path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            },
            copy_delay_config: Some(CopyDelayConfig {
                tasks: vec![
                    CopyDelayTask {
                        delay: Duration::hours(1),
                        first_delay: None,
                        increment: 2,
                        first_increment: None,
                        count: 1,
                        custom_rolling_stock: None,
                    },
                ],
            }),
        };

        let expected = vec![
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10001".into())
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 08:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: None,
            },
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10003".into())
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 09:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: None,
            },
        ];

        assert_eq!(
            generate_zug(&env, &prejoined_fpn_path, config).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(route_path).unwrap(), ROUTE_TRN);
        assert_eq!(fs::read_to_string(meta_data_path).unwrap(), META_DATA_TRN);
        assert_eq!(fs::read_to_string(rolling_stock_path).unwrap(), ROLLING_STOCK_TRN);
    }

    #[test]
    fn test_generate_zug_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let fpn_path = tmp_dir.path().join("test.fpn");
        let prejoined_fpn_path = PrejoinedZusiPath::new(&env.data_dir, ZusiPath::new_using_data_dir(fpn_path, &env.data_dir).unwrap());

        let route_path = tmp_dir.path().join("test/10001.trn");
        fs::create_dir_all(route_path.parent().unwrap()).unwrap();
        fs::write(&route_path, ROUTE_TRN_WITH_TIMETABLE).unwrap();

        let route_timetable_path = tmp_dir.path().join("test/10001.timetable.xml");
        fs::create_dir_all(route_timetable_path.parent().unwrap()).unwrap();
        fs::write(&route_timetable_path, ROUTE_TIMETABLE).unwrap();

        let meta_data_path = tmp_dir.path().join("test/dev/meta-data.trn");
        fs::create_dir_all(meta_data_path.parent().unwrap()).unwrap();
        fs::write(&meta_data_path, META_DATA_TRN).unwrap();

        let rolling_stock_path = tmp_dir.path().join("test/dev/rolling-stock/Triebwagen-A.trn");
        fs::create_dir_all(rolling_stock_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_path, ROLLING_STOCK_TRN_WITH_TIMETABLE).unwrap();

        let rolling_stock_timetable_path = tmp_dir.path().join("test/dev/rolling-stock/Triebwagen-A.timetable.xml");
        fs::create_dir_all(rolling_stock_timetable_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_timetable_path, ROLLING_STOCK_TIMETABLE).unwrap();

        let config = ZugConfig {
            nummer: "10001".into(),
            gattung: "RB".into(),
            meta_data: Some(MetaDataConfig {
                path: meta_data_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            }),
            route: RouteConfig {
                parts: vec![
                    RoutePart {
                        source: RoutePartSource::TrainFileByPath { path: route_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                        time_fix: None,
                        apply_schedule: None,
                    },
                ],
            },
            rolling_stock: RollingStockConfig {
                path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            },
            copy_delay_config: Some(CopyDelayConfig {
                tasks: vec![
                    CopyDelayTask {
                        delay: Duration::hours(1),
                        first_delay: None,
                        increment: 2,
                        first_increment: None,
                        count: 1,
                        custom_rolling_stock: None,
                    },
                ],
            }),
        };

        let expected = vec![
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10001".into())
                        .mindest_bremshundertstel(1.4)
                        .bremsstellung_zug(Bremsstellung::PMg)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 08:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: Some(TypedZusi::<Buchfahrplan>::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Buchfahrplan).version("A.4".into()).min_version("A.4".into()).build())
                    .value(Buchfahrplan::builder()
                        .datei_trn(Datei::builder().build())
                        .datei_fpn(Datei::builder().build())
                        .utm(UTM::builder().build())
                        .gattung("RB".into())
                        .nummer("10001".into())
                        .speed_max(20.)
                        .mindest_bremshundertstel(1.4)
                        .bremsstellung_zug(Bremsstellung::PMg)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_zeilen(vec![
                            FahrplanZeile::builder()
                                .fahrplan_laufweg(20092.018)
                                .fahrplan_km(vec![FahrplanKm::builder().km(32.8757).build()])
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Elze".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:39:00)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:41:40)).build()))
                                .build(),
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
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:45:00)).build()))
                                .build(),
                            FahrplanZeile::builder()
                                .fahrplan_regelgleis_gegengleis(1)
                                .fahrplan_laufweg(29134.139)
                                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:48:00)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:48:40)).build()))
                                .build(),
                        ])
                        .build())
                    .build()),
            },
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10003".into())
                        .mindest_bremshundertstel(1.4)
                        .bremsstellung_zug(Bremsstellung::PMg)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 09:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: Some(TypedZusi::<Buchfahrplan>::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Buchfahrplan).version("A.4".into()).min_version("A.4".into()).build())
                    .value(Buchfahrplan::builder()
                        .datei_trn(Datei::builder().build())
                        .datei_fpn(Datei::builder().build())
                        .utm(UTM::builder().build())
                        .gattung("RB".into())
                        .nummer("10003".into())
                        .speed_max(20.)
                        .mindest_bremshundertstel(1.4)
                        .bremsstellung_zug(Bremsstellung::PMg)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_zeilen(vec![
                            FahrplanZeile::builder()
                                .fahrplan_laufweg(20092.018)
                                .fahrplan_km(vec![FahrplanKm::builder().km(32.8757).build()])
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Elze".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 09:39:00)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:41:40)).build()))
                                .build(),
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
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:45:00)).build()))
                                .build(),
                            FahrplanZeile::builder()
                                .fahrplan_regelgleis_gegengleis(1)
                                .fahrplan_laufweg(29134.139)
                                .fahrplan_km(vec![FahrplanKm::builder().km(9.0405).build()])
                                .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Osterwald Hp".into()).build()))
                                .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 09:48:00)).build()))
                                .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 09:48:40)).build()))
                                .build(),
                        ])
                        .build())
                    .build()),
            },
        ];

        assert_eq!(
            generate_zug(&env, &prejoined_fpn_path, config).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(route_path).unwrap(), ROUTE_TRN_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(route_timetable_path).unwrap(), ROUTE_TIMETABLE);
        assert_eq!(fs::read_to_string(meta_data_path).unwrap(), META_DATA_TRN);
        assert_eq!(fs::read_to_string(rolling_stock_path).unwrap(), ROLLING_STOCK_TRN_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(rolling_stock_timetable_path).unwrap(), ROLLING_STOCK_TIMETABLE);
    }

    #[test]
    fn test_generate_zug_with_ignored_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let fpn_path = tmp_dir.path().join("test.fpn");
        let prejoined_fpn_path = PrejoinedZusiPath::new(&env.data_dir, ZusiPath::new_using_data_dir(fpn_path, &env.data_dir).unwrap());

        let route_path = tmp_dir.path().join("test/10001.trn");
        fs::create_dir_all(route_path.parent().unwrap()).unwrap();
        fs::write(&route_path, ROUTE_TRN_WITH_TIMETABLE).unwrap();

        let route_timetable_path = tmp_dir.path().join("test/10001.timetable.xml");
        fs::create_dir_all(route_timetable_path.parent().unwrap()).unwrap();
        fs::write(&route_timetable_path, ROUTE_TIMETABLE).unwrap();

        let meta_data_path = tmp_dir.path().join("test/dev/meta-data.trn");
        fs::create_dir_all(meta_data_path.parent().unwrap()).unwrap();
        fs::write(&meta_data_path, META_DATA_TRN).unwrap();

        let rolling_stock_path = tmp_dir.path().join("test/dev/rolling-stock/Triebwagen-A.trn");
        fs::create_dir_all(rolling_stock_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_path, ROLLING_STOCK_TRN).unwrap();

        let config = ZugConfig {
            nummer: "10001".into(),
            gattung: "RB".into(),
            meta_data: Some(MetaDataConfig {
                path: meta_data_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            }),
            route: RouteConfig {
                parts: vec![
                    RoutePart {
                        source: RoutePartSource::TrainFileByPath { path: route_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                        time_fix: None,
                        apply_schedule: None,
                    },
                ],
            },
            rolling_stock: RollingStockConfig {
                path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            },
            copy_delay_config: Some(CopyDelayConfig {
                tasks: vec![
                    CopyDelayTask {
                        delay: Duration::hours(1),
                        first_delay: None,
                        increment: 2,
                        first_increment: None,
                        count: 1,
                        custom_rolling_stock: None,
                    },
                ],
            }),
        };

        let expected = vec![
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10001".into())
                        .mindest_bremshundertstel(1.4)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 08:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 08:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 08:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: None,
            },
            GeneratedZug {
                zug: TypedZusi::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                    .value(Zug::builder()
                        .gattung("RB".into())
                        .nummer("10003".into())
                        .mindest_bremshundertstel(1.4)
                        .zuglauf("ADorf - BDorf".into())
                        .fahrplan_datei(Datei::builder().dateiname(prejoined_fpn_path.zusi_path().clone()).nur_info(true).build())
                        .fahrstrassen_name("Aufgleispunkt -> Hildesheim Hbf F".into())
                        .fahrplan_eintraege(vec![
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:39:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:41:40)))
                                .signal_vorlauf(180.)
                                .betriebsstelle("Elze".into())
                                .fahrplan_signal_eintraege(vec![
                                    FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                                ])
                                .build(),
                            FahrplanEintrag::builder()
                                .abfahrt(Some(datetime!(2024-06-20 09:45:00)))
                                .betriebsstelle("Mehle Hp".into())
                                .build(),
                            FahrplanEintrag::builder()
                                .ankunft(Some(datetime!(2024-06-20 09:48:00)))
                                .abfahrt(Some(datetime!(2024-06-20 09:48:40)))
                                .signal_vorlauf(160.)
                                .betriebsstelle("Osterwald Hp".into())
                                .build(),
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
                        .build())
                    .build(),
                buchfahrplan: None,
            },
        ];

        assert_eq!(
            generate_zug(&env, &prejoined_fpn_path, config).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(route_path).unwrap(), ROUTE_TRN_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(route_timetable_path).unwrap(), ROUTE_TIMETABLE);
        assert_eq!(fs::read_to_string(meta_data_path).unwrap(), META_DATA_TRN);
        assert_eq!(fs::read_to_string(rolling_stock_path).unwrap(), ROLLING_STOCK_TRN);
    }
}