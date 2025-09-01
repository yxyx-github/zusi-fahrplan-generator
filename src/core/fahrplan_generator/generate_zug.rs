mod generate_route;

use crate::core::fahrplan_generator::generate_zug::generate_route::{generate_route, GenerateRouteError};
use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::datei_from_zusi_path;
use crate::core::replace_rolling_stock::{replace_rolling_stock, ReplaceRollingStockError};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::ZugConfig;
use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::TypedZusi;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateZugError {
    GenerateRouteError {
        zug_nummer: String,
        error: GenerateRouteError,
    },
    AttachFahrplanFileError {
        error: FileError,
    },
    ApplyRollingStockError {
        error: ReplaceRollingStockError,
    },
}

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, zug_config: ZugConfig) -> Result<TypedZusi<Zug>, GenerateZugError> {
    let fahrplan_datei = datei_from_zusi_path(fahrplan_path.zusi_path(), true)
        .map_err(|error| GenerateZugError::AttachFahrplanFileError { error: (&zug_config.rolling_stock.path, error).into() })?;

    let route = generate_route(env, zug_config.route)
        .map_err(|error| GenerateZugError::GenerateRouteError {
            zug_nummer: zug_config.nummer.clone(), // TODO: do not clone
            error,
        })?;

    let zug = Zug::builder()
        .gattung(zug_config.gattung)
        .nummer(zug_config.nummer)
        .fahrplan_datei(fahrplan_datei)
        .fahrstrassen_name(route.aufgleis_fahrstrasse)
        .fahrplan_eintraege(route.fahrplan_eintraege)
        .fahrzeug_varianten(FahrzeugVarianten::builder().build())
        .build();

    let zug = replace_rolling_stock(env, zug_config.rolling_stock, zug)
        .map_err(|error| GenerateZugError::ApplyRollingStockError { error })?;

    Ok(
        TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(zug)
            .build()
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_zug() {

    }
}