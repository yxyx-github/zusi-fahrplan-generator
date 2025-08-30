use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::helpers::read_zug;
use crate::input::fahrplan_config::{ApplySchedule, RouteConfig, RoutePart, RoutePartSource};
use crate::input::ZusiEnvironment;
use std::path::PathBuf;
use serde_helpers::xml::FromXML;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
use crate::core::schedule;
use crate::input::schedule::Schedule;

pub struct ResolvedRoute {
    pub aufgleis_fahrstrasse: String,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
}

impl ResolvedRoute {
    fn new() -> Self {
        Self {
            aufgleis_fahrstrasse: "".into(),
            fahrplan_eintraege: vec![],
        }
    }
}

pub fn generate_route(env: &ZusiEnvironment, config: RouteConfig, zug_nummer: &str) -> Result<ResolvedRoute, GenerateFahrplanError> {
    let route_parts: Result<Vec<_>, _> = config.parts
        .into_iter()
        .map(|part| retrieve_route_part(env, part, zug_nummer))
        .collect();
    let route_parts = route_parts?;
    match route_parts.into_iter().reduce(|mut acc, mut part| {
        acc.fahrplan_eintraege.append(&mut part.fahrplan_eintraege); // TODO: check equality of entries first
        acc
    }) {
        None => Err(GenerateFahrplanError::NoRouteParts { zug_nummer: zug_nummer.into() }),
        Some(route) => Ok(route),
    }
}

fn retrieve_route_part(env: &ZusiEnvironment, part: RoutePart, zug_nummer: &str) -> Result<ResolvedRoute, GenerateFahrplanError> {
    let mut route_part = match part.source {
        RoutePartSource::TrainFileByPath { ref path } => retrieve_route_part_by_path(env, path),
        RoutePartSource::TrainConfigByNummer { .. } => todo!(),
    }?;
    if route_part.fahrplan_eintraege.is_empty() {
        Err(GenerateFahrplanError::EmptyRoutePart { source: part.source })
    } else {
        // TODO: override meta data
        if let Some(ApplySchedule { path, .. }) = part.apply_schedule {
            let schedule = Schedule::from_xml_file_by_path(&path).map_err(|error| (&path, error))?;
            schedule::apply(&mut route_part.fahrplan_eintraege, &schedule).map_err(|error| GenerateFahrplanError::InvalidSchedule {
                zug_nummer: zug_nummer.into(),
                path,
                error,
            })?;
        }
        // TODO: apply TimeFix
        Ok(route_part)
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