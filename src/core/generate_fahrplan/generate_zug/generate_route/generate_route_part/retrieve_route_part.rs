use std::path::PathBuf;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;
use crate::core::generate_fahrplan::generate_zug::generate_route::generate_route_part::GenerateRoutePartError;
use crate::core::generate_fahrplan::generate_zug::generate_route::resolved_route::{ResolvedRoutePart, RouteStartData};
use crate::core::lib::helpers::{override_with_non_default, read_buchfahrplan, read_zug};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::RoutePartSource;

// TODO: use own error type
pub fn retrieve_route_part(env: &ZusiEnvironment, source: RoutePartSource) -> Result<ResolvedRoutePart, GenerateRoutePartError> {
    match source { // TODO: extract
        RoutePartSource::TrainFileByPath { ref path } => retrieve_route_part_by_path(env, path),
        RoutePartSource::TrainConfigByNummer { .. } => todo!(),
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

// TODO: test