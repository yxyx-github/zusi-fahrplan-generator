pub mod generate_route_part;
pub mod merge_routes;
pub mod resolved_route;

use crate::core::generate_fahrplan::generate_zug::generate_route::generate_route_part::{generate_route_part, GenerateRoutePartError};
use crate::core::generate_fahrplan::generate_zug::generate_route::merge_routes::{merge_routes, MergeRoutePartsError};
use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::ResolvedRoute;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::{RouteConfig, RoutePartSource};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GenerateRouteError {
    #[error("The route part configuration for source {source:?} is invalid: {error}")]
    GenerateRoutePartError {
        source: RoutePartSource,

        #[source]
        error: GenerateRoutePartError,
    },

    #[error("No route parts were found but at least one is required.")]
    NoRouteParts,

    #[error("The route parts couldn't be mered: {error}")]
    MergeRoutePartsError {
        #[from]
        error: MergeRoutePartsError,
    }
}

pub fn generate_route(env: &ZusiEnvironment, config: RouteConfig) -> Result<ResolvedRoute, GenerateRouteError> {
    let mut resolved_route_parts = config.parts
        .into_iter()
        .map(|part| generate_route_part(env, part.clone()) // TODO: do not clone
            .map_err(|error| GenerateRouteError::GenerateRoutePartError {
                source: part.source,
                error,
            })
        ).collect::<Result<VecDeque<_>, _>>()?;
    let generated_route = resolved_route_parts.pop_front().ok_or(GenerateRouteError::NoRouteParts)?;
    resolved_route_parts
        .into_iter()
        .try_fold(
            generated_route,
            |generated_route, item|
                merge_routes(generated_route, item).map_err(GenerateRouteError::from)
        ).map(|route| route.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::RouteStartData;
    use crate::input::fahrplan_config::{ApplySchedule, RoutePart, RouteTimeFix, RouteTimeFixType};
    use std::fs;
    use tempfile::tempdir;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_km::FahrplanKm;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name_rechts::FahrplanNameRechts;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_signal_typ::FahrplanSignalTyp;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_v_max::FahrplanVMax;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
    use zusi_xml_lib::xml::zusi::lib::fahrplan_eintrag::FahrplanEintragsTyp;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
    use zusi_xml_lib::xml::zusi::zug::standort_modus::StandortModus;

    const TRN1: &str = r#"
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

    const TRN1_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="00001.timetable.xml"/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const TIMETABLE1: &str = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
            <Buchfahrplan Gattung="RB" Nummer="00001">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U" />
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

    const TRN2: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrplanEintrag Betrst="Voldagsen" FplEintrag="1">
                    <FahrplanSignalEintrag FahrplanSignal="A"/>
                </FahrplanEintrag>
                <FahrplanEintrag Ank="2024-06-20 08:52:10" Abf="2024-06-20 08:52:50" Signalvorlauf="160" Betrst="Voldagsen">
                    <FahrplanSignalEintrag FahrplanSignal="N2"/>
                </FahrplanEintrag>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const TRN2_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="00002.timetable.xml"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrplanEintrag Betrst="Voldagsen" FplEintrag="1">
                    <FahrplanSignalEintrag FahrplanSignal="A"/>
                </FahrplanEintrag>
                <FahrplanEintrag Ank="2024-06-20 08:52:10" Abf="2024-06-20 08:52:50" Signalvorlauf="160" Betrst="Voldagsen">
                    <FahrplanSignalEintrag FahrplanSignal="N2"/>
                </FahrplanEintrag>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const TIMETABLE2: &str = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
            <Buchfahrplan Gattung="RB" Nummer="00002">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U" />
                <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                    <Fplkm km="9.0405"/>
                    <FplName FplNameText="Osterwald Hp"/>
                    <FplAnk Ank="2024-06-20 08:48:00"/>
                    <FplAbf Abf="2024-06-20 08:48:40"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="32220.396">
                    <Fplkm km="12.128"/>
                    <FplSignaltyp FplSignaltypNr="7"/>
                    <FplNameRechts FplNameText="E 60"/>
                </FplZeile>
                <FplZeile FplLaufweg="32660.822">
                    <FplvMax vMax="33.3333"/>
                    <Fplkm km="12.5721"/>
                </FplZeile>
                <FplZeile FplLaufweg="32883.34">
                    <Fplkm km="12.7907"/>
                    <FplName FplNameText="Voldagsen"/>
                    <FplAnk Ank="2024-06-20 08:52:10"/>
                    <FplAbf Abf="2024-06-20 08:52:50"/>
                </FplZeile>
            </Buchfahrplan>
        </Zusi>
    "#;

    const EMPTY_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const SCHEDULE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Schedule>
            <ScheduleEntry Betriebsstelle="Elze" DrivingTime="00:02:20" StopTime="00:02:40"/>
            <ScheduleEntry Betriebsstelle="Mehle Hp" DrivingTime="00:03:20"/>
            <ScheduleEntry Betriebsstelle="Osterwald Hp" DrivingTime="00:03:00" StopTime="00:00:50"/>
            <ScheduleEntry Betriebsstelle="Voldagsen" DrivingTime="00:03:30" StopTime="00:00:40"/>
        </Schedule>
    "#;

    #[test]
    fn test_generate_route() {
        let tmp_dir = tempdir().unwrap();

        let trn1_path = tmp_dir.path().join("00001.trn");
        fs::write(&trn1_path, TRN1).unwrap();

        let trn2_path = tmp_dir.path().join("00002.trn");
        fs::write(&trn2_path, TRN2).unwrap();

        let schedule_path = tmp_dir.path().join("00000.schedule.xml");
        fs::write(&schedule_path, SCHEDULE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_config = RouteConfig {
            parts: vec![
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn1_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn2_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:49:50) }),
                    apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
                },
            ],
        };

        let expected = ResolvedRoute {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder()
                    .ankunft(Some(datetime!(2024-06-20 08:40:00)))
                    .abfahrt(Some(datetime!(2024-06-20 08:42:40)))
                    .signal_vorlauf(180.)
                    .betriebsstelle("Elze".into())
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                    ])
                    .build(),
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
                FahrplanEintrag::builder()
                    .betriebsstelle("Voldagsen".into())
                    .fahrplan_eintrag(FahrplanEintragsTyp::Hilfseintrag)
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    ])
                    .build(),
                FahrplanEintrag::builder()
                    .ankunft(Some(datetime!(2024-06-20 08:53:20)))
                    .abfahrt(Some(datetime!(2024-06-20 08:54:00)))
                    .signal_vorlauf(160.)
                    .betriebsstelle("Voldagsen".into())
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("N2".into()).build(),
                    ])
                    .build(),
            ],
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        let generated_route = generate_route(&env, route_config).unwrap();

        assert_eq!(generated_route, expected);

        assert_eq!(fs::read_to_string(trn1_path).unwrap(), TRN1);
        assert_eq!(fs::read_to_string(trn2_path).unwrap(), TRN2);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }

    #[test]
    fn test_cannot_generate_route_with_non_consecutive_route_parts() {
        let tmp_dir = tempdir().unwrap();

        let trn1_path = tmp_dir.path().join("00001.trn");
        fs::write(&trn1_path, TRN1).unwrap();

        let trn2_path = tmp_dir.path().join("00002.trn");
        fs::write(&trn2_path, TRN2).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_config = RouteConfig {
            parts: vec![
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn2_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn1_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
            ],
        };

        assert!(matches!(
            generate_route(&env, route_config).unwrap_err(),
            GenerateRouteError::MergeRoutePartsError { error: MergeRoutePartsError::NonConsecutiveRouteParts, .. },
        ));

        assert_eq!(fs::read_to_string(trn1_path).unwrap(), TRN1);
        assert_eq!(fs::read_to_string(trn2_path).unwrap(), TRN2);
    }

    #[test]
    fn test_cannot_generate_route_with_invalid_route_part() {
        let tmp_dir = tempdir().unwrap();

        let trn1_path = tmp_dir.path().join("00001.trn");
        fs::write(&trn1_path, TRN1).unwrap();

        let empty_trn_path = tmp_dir.path().join("00002.trn");
        fs::write(&empty_trn_path, EMPTY_TRN).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_config = RouteConfig {
            parts: vec![
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn1_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: empty_trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
            ],
        };

        assert!(matches!(
            generate_route(&env, route_config).unwrap_err(),
            GenerateRouteError::GenerateRoutePartError { error: GenerateRoutePartError::EmptyRoutePart, .. },
        ));

        assert_eq!(fs::read_to_string(trn1_path).unwrap(), TRN1);
        assert_eq!(fs::read_to_string(empty_trn_path).unwrap(), EMPTY_TRN);
    }

    #[test]
    fn test_cannot_generate_route_with_empty_route_parts() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_config = RouteConfig {
            parts: vec![],
        };

        assert_eq!(generate_route(&env, route_config).unwrap_err(), GenerateRouteError::NoRouteParts);
    }

    #[test]
    fn test_generate_route_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let trn1_path = tmp_dir.path().join("00001.trn");
        fs::write(&trn1_path, TRN1_WITH_TIMETABLE).unwrap();

        let timetable1_path = tmp_dir.path().join("00001.timetable.xml");
        fs::write(&timetable1_path, TIMETABLE1).unwrap();

        let trn2_path = tmp_dir.path().join("00002.trn");
        fs::write(&trn2_path, TRN2_WITH_TIMETABLE).unwrap();

        let timetable2_path = tmp_dir.path().join("00002.timetable.xml");
        fs::write(&timetable2_path, TIMETABLE2).unwrap();

        let schedule_path = tmp_dir.path().join("00000.schedule.xml");
        fs::write(&schedule_path, SCHEDULE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_config = RouteConfig {
            parts: vec![
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn1_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: None,
                    apply_schedule: None,
                },
                RoutePart {
                    source: RoutePartSource::TrainFileByPath { path: trn2_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                    time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:49:50) }),
                    apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
                },
            ],
        };

        let expected = ResolvedRoute {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: Some(0.0),
                gnt_spalte: Some(false),
            },
            fahrplan_eintraege: vec![
                FahrplanEintrag::builder()
                    .ankunft(Some(datetime!(2024-06-20 08:40:00)))
                    .abfahrt(Some(datetime!(2024-06-20 08:42:40)))
                    .signal_vorlauf(180.)
                    .betriebsstelle("Elze".into())
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("N1".into()).build(),
                    ])
                    .build(),
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
                FahrplanEintrag::builder()
                    .betriebsstelle("Voldagsen".into())
                    .fahrplan_eintrag(FahrplanEintragsTyp::Hilfseintrag)
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("A".into()).build(),
                    ])
                    .build(),
                FahrplanEintrag::builder()
                    .ankunft(Some(datetime!(2024-06-20 08:53:20)))
                    .abfahrt(Some(datetime!(2024-06-20 08:54:00)))
                    .signal_vorlauf(160.)
                    .betriebsstelle("Voldagsen".into())
                    .fahrplan_signal_eintraege(vec![
                        FahrplanSignalEintrag::builder().fahrplan_signal("N2".into()).build(),
                    ])
                    .build(),
            ],
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(20092.018)
                    .fahrplan_km(vec![FahrplanKm::builder().km(32.8757).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Elze".into()).build()))
                    .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:40:00)).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:42:40)).build()))
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
                    .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2024-06-20 08:53:20)).build()))
                    .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2024-06-20 08:54:00)).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };

        let generated_route = generate_route(&env, route_config).unwrap();

        assert_eq!(generated_route, expected);

        assert_eq!(fs::read_to_string(trn1_path).unwrap(), TRN1_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(timetable1_path).unwrap(), TIMETABLE1);
        assert_eq!(fs::read_to_string(trn2_path).unwrap(), TRN2_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(timetable2_path).unwrap(), TIMETABLE2);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }
}