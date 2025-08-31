pub mod generate_route_part;
pub mod merge_routes;

use crate::core::fahrplan_generator::generate_zug::generate_route::generate_route_part::{generate_route_part, GenerateRoutePartError};
use crate::core::fahrplan_generator::generate_zug::generate_route::merge_routes::{merge_routes, RoutesCanNotBeMerged};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::{RouteConfig, RoutePartSource};
use std::collections::VecDeque;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateRouteError {
    GenerateRoutePartError {
        source: RoutePartSource,
        error: GenerateRoutePartError,
    },
    NoRouteParts,
    RoutePartsCanNotBeMerged,
}

impl From<RoutesCanNotBeMerged> for GenerateRouteError {
    fn from(_: RoutesCanNotBeMerged) -> Self {
        GenerateRouteError::RoutePartsCanNotBeMerged
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoute {
    pub aufgleis_fahrstrasse: String,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
}
pub fn generate_route(env: &ZusiEnvironment, config: RouteConfig) -> Result<ResolvedRoute, GenerateRouteError> {
    let mut resolved_route_parts = config.parts
        .into_iter()
        .map(|part| generate_route_part(env, part.clone())
            .map_err(|error| GenerateRouteError::GenerateRoutePartError { // TODO: do not clone
                source: part.source,
                error,
            }))
        .collect::<Result<VecDeque<_>, _>>()?;
    let generated_route = resolved_route_parts.pop_front().ok_or(GenerateRouteError::NoRouteParts)?;
    resolved_route_parts
        .into_iter()
        .try_fold(
            generated_route,
            |generated_route, item|
                merge_routes(generated_route, item).map_err(GenerateRouteError::from)
        )
}