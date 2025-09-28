use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::{ResolvedRoutePart, RouteStartData};
use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::{delay_fahrplan_eintraege, override_with_non_default, read_buchfahrplan, read_zug};
use crate::core::schedules::apply::{apply_schedule, ApplyScheduleError};
use crate::core::schedules::update_buchfahrplan::{update_buchfahrplan, UpdateBuchfahrplanError};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::{ApplySchedule, RoutePart, RoutePartSource, RouteTimeFix, RouteTimeFixType};
use crate::input::schedule::Schedule;
use serde_helpers::xml::FromXML;
use std::path::PathBuf;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GenerateRoutePartError {
    #[error("The route part must contain at least one FahrplanEintrag.")]
    EmptyRoutePart,

    #[error("The schedule couldn't be read: {error}")]
    ReadScheduleError {
        #[source]
        error: FileError,
    },

    #[error("Couldn't apply the schedule: {error}")]
    CouldNotApplySchedule {
        #[from]
        error: ApplyScheduleError,
    },

    /// Occours if corresponding time is [None].
    #[error("The time fix couldn't be applied to the route part. This is likely because the selected FahrplanEintrag entry doesn't contain a value Abfahrt or Ankunft.")]
    CouldNotApplyTimeFix,

    #[error("The route source couldn't be read: {error}")]
    ReadRouteError {
        #[source]
        error: FileError, // TODO: won't be always a FileError, e.g. if TrainConfigByNummer will be implemented
    },

    #[error("The Buchfahrplan attached to the route couldn't be read: {error}")]
    ReadBuchfahrplanError {
        #[source]
        error: FileError,
    },

    #[error("The Buchfahrplan attached to the route couldn't be updated: {error}")]
    UpdateBuchfahrplanError {
        #[from]
        error: UpdateBuchfahrplanError,
    },
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
                .map_err(|error| GenerateRoutePartError::ReadScheduleError { error })?;
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
            delay_fahrplan_eintraege(&mut resolved_route_part.fahrplan_eintraege, time_fix_diff);
            resolved_route_part.has_time_fix = true;
        }

        if !resolved_route_part.fahrplan_zeilen.is_empty() {
            update_buchfahrplan(&resolved_route_part.fahrplan_eintraege, &mut resolved_route_part.fahrplan_zeilen)?;
        }

        resolved_route_part.start_data.fahrzeug_verband_aktion = route_part.start_fahrzeug_verband_aktion;

        Ok(resolved_route_part)
    }
}

fn retrieve_route_part_by_path(env: &ZusiEnvironment, path: &PathBuf) -> Result<ResolvedRoutePart, GenerateRoutePartError> {
    let path = env.path_to_prejoined_zusi_path(path)
        .map_err(|error| GenerateRoutePartError::ReadRouteError { error })?;
    let mut route_template = read_zug(path.full_path())
        .map_err(|error| GenerateRoutePartError::ReadRouteError { error })?.value;

    let (fahrplan_zeilen, km_start, gnt_spalte) = if let Some(Datei { dateiname, .. }) = route_template.buchfahrplan_roh_datei {
        let buchfahrplan_path = env.zusi_path_to_prejoined_zusi_path(dateiname);
        let buchfahrplan_template = read_buchfahrplan(buchfahrplan_path.full_path())
            .map_err(|error| GenerateRoutePartError::ReadBuchfahrplanError { error })?.value;
        override_with_non_default(&mut route_template.mindest_bremshundertstel, buchfahrplan_template.mindest_bremshundertstel);
        (buchfahrplan_template.fahrplan_zeilen, Some(buchfahrplan_template.km_start), Some(buchfahrplan_template.gnt_spalte))
    } else {
        (vec![], None, None)
    };

    Ok(
        ResolvedRoutePart::new(
            RouteStartData {
                aufgleis_fahrstrasse: route_template.fahrstrassen_name,
                standort_modus: route_template.standort_modus,
                start_vorschubweg: route_template.start_vorschubweg,
                speed_anfang: route_template.speed_anfang,
                km_start,
                gnt_spalte,
                fahrzeug_verband_aktion: None,
            },
            route_template.fahrplan_eintraege,
            fahrplan_zeilen,
            route_template.mindest_bremshundertstel,
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
    use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrzeug_verband_aktion::FahrzeugVerbandAktion;
    use zusi_xml_lib::xml::zusi::zug::standort_modus::StandortModus;
    use crate::core::lib::file_error::FileErrorKind;
    use crate::input::fahrplan_config::StartFahrzeugVerbandAktion;

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
            start_fahrzeug_verband_aktion: Some(StartFahrzeugVerbandAktion {
                aktion: FahrzeugVerbandAktion::Fueherstandswechsel,
                wende_signal_abstand: 0.,
            }),
            time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:42:40) }),
            apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
        };

        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: None,
                gnt_spalte: None,
                fahrzeug_verband_aktion: Some(StartFahrzeugVerbandAktion {
                    aktion: FahrzeugVerbandAktion::Fueherstandswechsel,
                    wende_signal_abstand: 0.,
                }),
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
            has_time_fix: true,
            fahrplan_zeilen: vec![],
            mindest_bremshundertstel: 0.,
        };

        let resolved_route_part = generate_route_part(&env, route_part).unwrap();

        assert_eq!(resolved_route_part, expected);

        assert_eq!(fs::read_to_string(trn_path).unwrap(), TRN);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }

    #[test]
    fn test_generate_route_part_with_non_existing_trn_file() {
        let tmp_dir = tempdir().unwrap();

        let trn_path = tmp_dir.path().join("00000.trn");

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_part = RoutePart {
            source: RoutePartSource::TrainFileByPath { path: trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
            start_fahrzeug_verband_aktion: None,
            time_fix: None,
            apply_schedule: None,
        };

        assert!(matches!(
            generate_route_part(&env, route_part).unwrap_err(),
            GenerateRoutePartError::ReadRouteError {
                error: FileError {
                    kind: FileErrorKind::IOError { .. },
                    ..
                },
            },
        ));
    }

    #[test]
    fn test_generate_route_part_with_buchfahrplan() {
        const TRN: &str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <Zusi>
                <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
                <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                    <Datei/>
                    <BuchfahrplanRohDatei Dateiname="00000.timetable.xml"/>
                    <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                    <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                    <FahrzeugVarianten/>
                </Zug>
            </Zusi>
        "#;

        const TIMETABLE: &str = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <Zusi>
                <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
                <Buchfahrplan Gattung="RB" Nummer="21041" kmStart="3.4">
                    <Datei_fpn/>
                    <Datei_trn/>
                    <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U" />
                    <FplZeile FplRglGgl="1" FplLaufweg="21799.445">
                        <FplvMax vMax="33.3333" />
                        <Fplkm km="1.7792" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="24631.027">
                        <Fplkm km="4.5357" />
                        <FplName FplNameText="Mehle Hp" />
                        <FplAbf Abf="2024-06-20 08:45:00" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                        <Fplkm km="9.0405" />
                        <FplName FplNameText="Osterwald Hp" />
                        <FplAnk Ank="2024-06-20 08:48:00" />
                        <FplAbf Abf="2024-06-20 08:48:40" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="32220.396">
                        <Fplkm km="12.128" />
                        <FplSignaltyp FplSignaltypNr="7" />
                        <FplNameRechts FplNameText="E 60" />
                    </FplZeile>
                </Buchfahrplan>
            </Zusi>
        "#;

        let tmp_dir = tempdir().unwrap();

        let trn_path = tmp_dir.path().join("00000.trn");
        fs::write(&trn_path, TRN).unwrap();

        let timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&timetable_path, TIMETABLE).unwrap();

        let schedule_path = tmp_dir.path().join("00000.schedule.xml");
        fs::write(&schedule_path, SCHEDULE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_part = RoutePart {
            source: RoutePartSource::TrainFileByPath { path: trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
            start_fahrzeug_verband_aktion: None,
            time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:46:00) }),
            apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
        };

        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: Some(3.4),
                gnt_spalte: Some(false),
                fahrzeug_verband_aktion: None,
            },
            fahrplan_eintraege: vec![
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
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![
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
            ],
            mindest_bremshundertstel: 0.,
        };

        let resolved_route_part = generate_route_part(&env, route_part).unwrap();

        assert_eq!(resolved_route_part, expected);

        assert_eq!(fs::read_to_string(trn_path).unwrap(), TRN);
        assert_eq!(fs::read_to_string(timetable_path).unwrap(), TIMETABLE);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }

    #[test]
    fn test_generate_route_part_with_buchfahrplan_without_first_ankunft_and_last_abfahrt() {
        const TRN: &str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <Zusi>
                <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
                <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                    <Datei/>
                    <BuchfahrplanRohDatei Dateiname="00000.timetable.xml"/>
                    <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                        <FahrplanSignalEintrag FahrplanSignal="N1"/>
                    </FahrplanEintrag>
                    <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                    <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                    <FahrzeugVarianten/>
                </Zug>
            </Zusi>
        "#;

        const TIMETABLE: &str = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <Zusi>
                <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0"/>
                <Buchfahrplan Gattung="RB" Nummer="21041" kmStart="3.4">
                    <Datei_fpn/>
                    <Datei_trn/>
                    <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
                    <FplZeile FplLaufweg="20092.018">
                        <Fplkm km="32.8757" />
                        <FplName FplNameText="Elze" />
                        <FplAbf Abf="2024-06-20 07:41:40" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="21799.445">
                        <FplvMax vMax="33.3333" />
                        <Fplkm km="1.7792" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="24631.027">
                        <Fplkm km="4.5357" />
                        <FplName FplNameText="Mehle Hp" />
                        <FplAbf Abf="2024-06-20 07:45:00" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                        <Fplkm km="9.0405" />
                        <FplName FplNameText="Osterwald Hp" />
                        <FplAnk Ank="2024-06-20 07:48:00" />
                    </FplZeile>
                    <FplZeile FplRglGgl="1" FplLaufweg="32220.396">
                        <Fplkm km="12.128" />
                        <FplSignaltyp FplSignaltypNr="7" />
                        <FplNameRechts FplNameText="E 60" />
                    </FplZeile>
                </Buchfahrplan>
            </Zusi>
        "#;

        let tmp_dir = tempdir().unwrap();

        let trn_path = tmp_dir.path().join("00000.trn");
        fs::write(&trn_path, TRN).unwrap();

        let timetable_path = tmp_dir.path().join("00000.timetable.xml");
        fs::write(&timetable_path, TIMETABLE).unwrap();

        let schedule_path = tmp_dir.path().join("00000.schedule.xml");
        fs::write(&schedule_path, SCHEDULE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_part = RoutePart {
            source: RoutePartSource::TrainFileByPath { path: trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
            start_fahrzeug_verband_aktion: None,
            time_fix: Some(RouteTimeFix { fix_type: RouteTimeFixType::StartAbf, value: datetime!(2024-06-20 08:42:40) }),
            apply_schedule: Some(ApplySchedule { path: schedule_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() }),
        };

        let expected = ResolvedRoutePart {
            start_data: RouteStartData {
                aufgleis_fahrstrasse: "Aufgleispunkt -> Hildesheim Hbf F".into(),
                standort_modus: StandortModus::Automatisch,
                start_vorschubweg: 0.0,
                speed_anfang: 0.0,
                km_start: Some(3.4),
                gnt_spalte: Some(false),
                fahrzeug_verband_aktion: None,
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
            ],
            has_time_fix: true,
            fahrplan_zeilen: vec![
                FahrplanZeile::builder()
                    .fahrplan_laufweg(20092.018)
                    .fahrplan_km(vec![FahrplanKm::builder().km(32.8757).build()])
                    .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Elze".into()).build()))
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
                    .build(),
                FahrplanZeile::builder()
                    .fahrplan_regelgleis_gegengleis(1)
                    .fahrplan_laufweg(32220.396)
                    .fahrplan_km(vec![FahrplanKm::builder().km(12.128).build()])
                    .fahrplan_signal_typ(Some(FahrplanSignalTyp::builder().fahrplan_signal_typ_nummer(7).build()))
                    .fahrplan_name_rechts(Some(FahrplanNameRechts::builder().fahrplan_name_text("E 60".into()).build()))
                    .build(),
            ],
            mindest_bremshundertstel: 0.,
        };

        let resolved_route_part = generate_route_part(&env, route_part).unwrap();

        assert_eq!(resolved_route_part, expected);

        assert_eq!(fs::read_to_string(trn_path).unwrap(), TRN);
        assert_eq!(fs::read_to_string(timetable_path).unwrap(), TIMETABLE);
        assert_eq!(fs::read_to_string(schedule_path).unwrap(), SCHEDULE);
    }

    #[test]
    fn test_generate_route_part_with_non_existing_buchfahrplan() {
        const TRN: &str = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <Zusi>
                <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
                <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                    <Datei/>
                    <BuchfahrplanRohDatei Dateiname="00000.timetable.xml"/>
                    <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                    <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                    <FahrzeugVarianten/>
                </Zug>
            </Zusi>
        "#;

        let tmp_dir = tempdir().unwrap();

        let trn_path = tmp_dir.path().join("00000.trn");
        fs::write(&trn_path, TRN).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let route_part = RoutePart {
            source: RoutePartSource::TrainFileByPath { path: trn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
            start_fahrzeug_verband_aktion: None,
            time_fix: None,
            apply_schedule: None,
        };

        assert!(matches!(
            generate_route_part(&env, route_part).unwrap_err(),
            GenerateRoutePartError::ReadBuchfahrplanError {
                error: FileError {
                    kind: FileErrorKind::IOError { .. },
                    ..
                },
            },
        ));

        assert_eq!(fs::read_to_string(trn_path).unwrap(), TRN);
    }
}