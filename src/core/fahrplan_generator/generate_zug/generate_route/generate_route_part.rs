use crate::core::lib::file_error::FileError;
use crate::core::fahrplan_generator::generate_zug::generate_route::resolved_route::ResolvedRoutePart;
use crate::core::lib::helpers::read_zug;
use crate::core::schedules::apply::{apply_schedule, ApplyScheduleError};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::{ApplySchedule, RoutePart, RoutePartSource, RouteTimeFix, RouteTimeFixType};
use crate::input::schedule::Schedule;
use serde_helpers::xml::FromXML;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateRoutePartError {
    EmptyRoutePart,
    CouldNotApplySchedule {
        error: ApplyScheduleError,
    },
    /// Occours if corresponding time is [None].
    CouldNotApplyTimeFix,
    ReadRouteError {
        error: FileError, // TODO: won't be always a FileError, e.g. if TrainConfigByNummer will be implemented
    },
}

impl From<ApplyScheduleError> for GenerateRoutePartError {
    fn from(error: ApplyScheduleError) -> Self {
        GenerateRoutePartError::CouldNotApplySchedule {error }
    }
}

pub fn generate_route_part(env: &ZusiEnvironment, route_part: RoutePart) -> Result<ResolvedRoutePart, GenerateRoutePartError> {
    let mut resolved_route_part = match route_part.source {
        RoutePartSource::TrainFileByPath { ref path } => retrieve_route_part_by_path(env, path),
        RoutePartSource::TrainConfigByNummer { .. } => todo!(),
    }?;
    if resolved_route_part.fahrplan_eintraege.is_empty() {
        Err(GenerateRoutePartError::EmptyRoutePart)
    } else {
        if let Some(ApplySchedule { path, .. }) = route_part.apply_schedule {
            let prejoined_path = env.path_to_prejoined_zusi_path(&path)
                .map_err(|error| GenerateRoutePartError::ReadRouteError { error: (&path, error).into() })?;
            let schedule = Schedule::from_xml_file_by_path(prejoined_path.full_path())
                .map_err(|error| GenerateRoutePartError::ReadRouteError { error: (prejoined_path.full_path(), error).into() })?;
            apply_schedule(&mut resolved_route_part.fahrplan_eintraege, &schedule)?;
        }
        if let Some(RouteTimeFix { fix_type, value }) = route_part.time_fix {
            let time_fix_diff = match fix_type {
                RouteTimeFixType::StartAbf => resolved_route_part.fahrplan_eintraege.first().and_then(|e| e.abfahrt),
                RouteTimeFixType::EndAnk => resolved_route_part.fahrplan_eintraege.last().and_then(|e| e.ankunft),
            }.map(|time| value - time)
                .ok_or(GenerateRoutePartError::CouldNotApplyTimeFix)?;
            resolved_route_part.fahrplan_eintraege.iter_mut().for_each(|fahrplan_eintrag| {
                fahrplan_eintrag.ankunft = fahrplan_eintrag.ankunft.map(|ankunft| ankunft + time_fix_diff);
                fahrplan_eintrag.abfahrt = fahrplan_eintrag.abfahrt.map(|abfahrt| abfahrt + time_fix_diff);
            });
            resolved_route_part.has_time_fix = true;
        }
        Ok(resolved_route_part)
    }
}

fn retrieve_route_part_by_path(env: &ZusiEnvironment, path: &PathBuf) -> Result<ResolvedRoutePart, GenerateRoutePartError> {
    let path = env.path_to_prejoined_zusi_path(path)
        .map_err(|error| GenerateRoutePartError::ReadRouteError { error: (&path, error).into() })?;
    let route_template = read_zug(path.full_path())
        .map_err(|error| GenerateRoutePartError::ReadRouteError { error })?;
    Ok(
        ResolvedRoutePart::new(route_template.value.fahrstrassen_name, route_template.value.fahrplan_eintraege)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::lib::fahrplan_eintrag::FahrplanEintragsTyp;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrplan_signal_eintrag::FahrplanSignalEintrag;
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

    const TRN: &str = r#"
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
    fn test_generate_route_part() {
        let tmp_dir = tempdir().unwrap();

        let trn_path = tmp_dir.path().join("00000.trn");
        fs::write(&trn_path, TRN).unwrap();

        let schedule_path = tmp_dir.path().join("00000.schedule.xml");
        fs::write(&schedule_path, SCHEDULE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_part = RoutePart {
            source: RoutePartSource::TrainFileByPath { path: trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
            time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:42:40) }),
            apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
        };

        let expected = ResolvedRoutePart {
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
            has_time_fix: true,
        };

        let resolved_route_part = generate_route_part(&env, route_part).unwrap();

        assert_eq!(resolved_route_part, expected);

        assert_eq!(fs::read_to_string(trn_path).unwrap(), TRN);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }
}