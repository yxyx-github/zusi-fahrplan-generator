mod generate_route;
mod add_meta_data;

use crate::core::copy_delay::{copy_delay, CopyDelayError};
use crate::core::generate_fahrplan::generate_zug::add_meta_data::{add_meta_data, AddMetaDataError};
use crate::core::generate_fahrplan::generate_zug::generate_route::{generate_route, GenerateRouteError};
use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::datei_from_prejoined_zusi_path;
use crate::core::replace_rolling_stock::{replace_rolling_stock, ReplaceRollingStockError};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::ZugConfig;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::TypedZusi;

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

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, zug_config: ZugConfig) -> Result<Vec<TypedZusi<Zug>>, GenerateZugError> {
    let zug_nummer = &zug_config.nummer;

    let fahrplan_datei = datei_from_prejoined_zusi_path(fahrplan_path, true)
        .map_err(|error| GenerateZugError::from((zug_nummer, GenerateZugErrorKind::AttachFahrplanFileError { error })))?;

    let route = generate_route(env, zug_config.route)
        .map_err(|error| GenerateZugError::from((zug_nummer, error.into())))?;

    let mut zug = Zug::builder()
        .gattung(zug_config.gattung)
        .nummer(zug_nummer.to_owned()) // TODO: do not clone
        .fahrplan_datei(fahrplan_datei)
        .fahrstrassen_name(route.aufgleis_fahrstrasse)
        .fahrplan_eintraege(route.fahrplan_eintraege)
        .fahrzeug_varianten(FahrzeugVarianten::builder().build())
        .build();

    if let Some(meta_data) = zug_config.meta_data {
        add_meta_data(env, meta_data, &mut zug)
            .map_err(|error| GenerateZugError::from((zug_nummer, error.into())))?;
    }

    replace_rolling_stock(env, zug_config.rolling_stock, &mut zug)
        .map_err(|error| GenerateZugError::from((zug_nummer, error.into())))?;

    let mut zuege = vec![zug];

    if let Some(copy_delay_config) = zug_config.copy_delay_config {
        let mut additional = copy_delay(env, copy_delay_config, zuege.first().unwrap())
            .map_err(|error| GenerateZugError::from((zug_nummer, error.into())))?;
        zuege.append(&mut additional);
    }

    let zuege = zuege
        .into_iter()
        .map(|zug| TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(zug)
            .build())
        .collect();

    Ok(zuege)
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
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPath;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::fahrzeug_info::FahrzeugInfo;

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
                        count: 1,
                        increment: 2,
                        custom_rolling_stock: None,
                    },
                ],
            }),
        };

        let expected = vec![
            TypedZusi::builder()
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
            TypedZusi::builder()
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
        ];

        assert_eq!(
            generate_zug(&env, &prejoined_fpn_path, config).unwrap(),
            expected,
        );

        assert_eq!(fs::read_to_string(route_path).unwrap(), ROUTE_TRN);
        assert_eq!(fs::read_to_string(meta_data_path).unwrap(), META_DATA_TRN);
        assert_eq!(fs::read_to_string(rolling_stock_path).unwrap(), ROLLING_STOCK_TRN);
    }
}