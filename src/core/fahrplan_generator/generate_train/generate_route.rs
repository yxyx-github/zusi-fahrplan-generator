pub mod generate_route_part;

use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::generate_train::generate_route::generate_route_part::generate_route_part;
use crate::input::fahrplan_config::RouteConfig;
use crate::input::ZusiEnvironment;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

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
    let resolved_route_parts: Result<Vec<_>, _> = config.parts
        .into_iter()
        .map(|part| generate_route_part(env, part, zug_nummer))
        .collect();
    let resolved_route_parts = resolved_route_parts?;
    match resolved_route_parts.into_iter().reduce(|mut acc, mut part| {
        acc.fahrplan_eintraege.append(&mut part.fahrplan_eintraege); // TODO: check equality of entries first
        acc
    }) {
        None => Err(GenerateFahrplanError::NoRouteParts { zug_nummer: zug_nummer.into() }),
        Some(route) => Ok(route),
    }
}