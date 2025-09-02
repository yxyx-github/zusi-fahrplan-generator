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
use crate::core::copy_delay::{copy_delay, CopyDelayError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateZugError { // TODO: zug_nummer should be available for all errors
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
    CopyDelayError {
        error: CopyDelayError,
    },
}

impl From<CopyDelayError> for GenerateZugError {
    fn from(error: CopyDelayError) -> Self {
        GenerateZugError::CopyDelayError { error }
    }
}

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, zug_config: ZugConfig) -> Result<Vec<TypedZusi<Zug>>, GenerateZugError> {
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

    let mut zuege = vec![zug];

    if let Some(copy_delay_config) = zug_config.copy_delay_config {
        let mut additional = copy_delay(env, copy_delay_config, zuege.first().unwrap())?;
        zuege.append(&mut additional);
    }

    let zuege = zuege
        .into_iter()
        .map(|zug| TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(zug)
            .build())
        .collect();

    Ok(zuege)
}

#[cfg(test)]
mod tests {
    
    #[test]
    fn test_generate_zug() {

    }
}