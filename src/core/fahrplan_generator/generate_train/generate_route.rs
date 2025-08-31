pub mod generate_route_part;
pub mod merge_route_parts;

use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::generate_train::generate_route::generate_route_part::generate_route_part;
use crate::core::fahrplan_generator::generate_train::generate_route::merge_route_parts::merge_routes;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::RouteConfig;
use std::collections::VecDeque;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoute {
    pub aufgleis_fahrstrasse: String,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
}
pub fn generate_route(env: &ZusiEnvironment, config: RouteConfig, zug_nummer: &str) -> Result<ResolvedRoute, GenerateFahrplanError> {
    let mut resolved_route_parts = config.parts
        .into_iter()
        .map(|part| generate_route_part(env, part, zug_nummer))
        .collect::<Result<VecDeque<_>, _>>()?;
    let generated_route = resolved_route_parts.pop_front().ok_or(GenerateFahrplanError::NoRouteParts { zug_nummer: zug_nummer.into() })?;
    resolved_route_parts
        .into_iter()
        .try_fold(
            generated_route,
            |generated_route, item|
                merge_routes(generated_route, item)
                    .map_err(|_| GenerateFahrplanError::RoutePartsCanNotBeMerged { zug_nummer: zug_nummer.into() })
        )
}