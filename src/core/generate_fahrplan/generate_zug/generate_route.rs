pub mod generate_route_part;
pub mod merge_routes;
mod resolved_route;

use crate::core::generate_fahrplan::generate_zug::generate_route::generate_route_part::{generate_route_part, GenerateRoutePartError};
use crate::core::generate_fahrplan::generate_zug::generate_route::merge_routes::{merge_routes, MergeRoutePartsError};
use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::ResolvedRoute;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::{RouteConfig, RoutePartSource};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
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
        .map(|part| generate_route_part(env, part.clone())
            .map_err(|error| GenerateRouteError::GenerateRoutePartError { // TODO: do not clone
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
    use crate::input::fahrplan_config::{ApplySchedule, RoutePart, RouteTimeFix, RouteTimeFixType};
    use std::fs;
    use tempfile::tempdir;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::lib::fahrplan_eintrag::FahrplanEintragsTyp;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

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
            aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
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
}