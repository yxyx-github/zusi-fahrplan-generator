mod generate_route;

use crate::core::fahrplan_generator::file_error::FileError;
use crate::core::fahrplan_generator::generate_zug::generate_route::{generate_route, GenerateRouteError};
use crate::core::fahrplan_generator::helpers::{datei_from_zusi_path, read_zug};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::ZugConfig;
use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
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
    ReadRollingStockError {
        error: FileError,
    },
}

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, train_config: ZugConfig) -> Result<TypedZusi<Zug>, GenerateZugError> {
    let fahrplan_datei = datei_from_zusi_path(fahrplan_path.zusi_path(), true)
        .map_err(|error| GenerateZugError::AttachFahrplanFileError { error: (&train_config.rolling_stock.path, error).into() })?;

    let rolling_stock_template_path = env.path_to_prejoined_zusi_path(&train_config.rolling_stock.path)
        .map_err(|error| GenerateZugError::ReadRollingStockError { error: (&train_config.rolling_stock.path, error).into() })?;
    let rolling_stock_template = read_zug(rolling_stock_template_path.full_path())
        .map_err(|error| GenerateZugError::ReadRollingStockError { error })?;

    let route = generate_route(env, train_config.route)
        .map_err(|error| GenerateZugError::GenerateRouteError {
            zug_nummer: train_config.nummer.clone(), // TODO: do not clone
            error,
        })?;

    let zug = Zug::builder()
        .gattung(train_config.gattung)
        .nummer(train_config.nummer)
        .fahrplan_datei(fahrplan_datei)
        .fahrstrassen_name(route.aufgleis_fahrstrasse)
        .fahrplan_eintraege(route.fahrplan_eintraege)
        .fahrzeug_varianten(rolling_stock_template.value.fahrzeug_varianten)
        .mindest_bremshundertstel(rolling_stock_template.value.mindest_bremshundertstel)
        .build();

    Ok(
        TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(zug)
            .build()
    )
}