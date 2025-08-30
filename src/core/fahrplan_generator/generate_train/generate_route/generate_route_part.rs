use std::path::PathBuf;
use serde_helpers::xml::FromXML;
use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::generate_train::generate_route::ResolvedRoute;
use crate::core::fahrplan_generator::helpers::read_zug;
use crate::core::schedules::apply::apply_schedule;
use crate::input::fahrplan_config::{ApplySchedule, RoutePart, RoutePartSource, RouteTimeFix, RouteTimeFixType};
use crate::input::schedule::Schedule;
use crate::input::ZusiEnvironment;

pub fn generate_route_part(env: &ZusiEnvironment, route_part: RoutePart, zug_nummer: &str) -> Result<ResolvedRoute, GenerateFahrplanError> {
    let mut resolved_route_part = match route_part.source {
        RoutePartSource::TrainFileByPath { ref path } => retrieve_route_part_by_path(env, path),
        RoutePartSource::TrainConfigByNummer { .. } => todo!(),
    }?;
    if resolved_route_part.fahrplan_eintraege.is_empty() {
        Err(GenerateFahrplanError::EmptyRoutePart { source: route_part.source })
    } else {
        // TODO: override meta data
        if let Some(ApplySchedule { path, .. }) = route_part.apply_schedule {
            let schedule = Schedule::from_xml_file_by_path(&path).map_err(|error| (&path, error))?;
            apply_schedule(&mut resolved_route_part.fahrplan_eintraege, &schedule).map_err(|error| GenerateFahrplanError::CouldNotApplySchedule {
                zug_nummer: zug_nummer.into(),
                path,
                error,
            })?;
        }
        if let Some(RouteTimeFix { fix_type, value }) = route_part.time_fix {
            let time_fix_diff = match fix_type {
                RouteTimeFixType::StartAbf => resolved_route_part.fahrplan_eintraege.first().and_then(|e| e.abfahrt),
                RouteTimeFixType::EndAnk => resolved_route_part.fahrplan_eintraege.last().and_then(|e| e.ankunft),
            }.map(|time| value - time)
                .ok_or(GenerateFahrplanError::CouldNotApplyTimeFix { zug_nummer: zug_nummer.into() })?;
            resolved_route_part.fahrplan_eintraege.iter_mut().for_each(|fahrplan_eintrag| {
                fahrplan_eintrag.ankunft.map(|ankunft| ankunft + time_fix_diff);
                fahrplan_eintrag.abfahrt.map(|abfahrt| abfahrt + time_fix_diff);
            });
        }
        Ok(resolved_route_part)
    }
}

fn retrieve_route_part_by_path(env: &ZusiEnvironment, path: &PathBuf) -> Result<ResolvedRoute, GenerateFahrplanError> {
    let path = env.path_to_prejoined_zusi_path(path).map_err(|error| (path, error))?;
    let route_template = read_zug(path.full_path())?;
    Ok(ResolvedRoute {
        aufgleis_fahrstrasse: route_template.value.fahrstrassen_name,
        fahrplan_eintraege: route_template.value.fahrplan_eintraege,
    })
}