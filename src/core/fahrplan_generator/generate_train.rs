use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::helpers::{datei_from_zusi_path, read_zug};
use crate::input::fahrplan_config::TrainConfig;
use crate::input::ZusiEnvironment;
use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::TypedZusi;

pub fn generate_zug(env: &ZusiEnvironment, fahrplan_path: &PrejoinedZusiPath, train_config: TrainConfig) -> Result<TypedZusi<Zug>, GenerateFahrplanError> {
    let fahrplan_datei = datei_from_zusi_path(fahrplan_path.zusi_path(), true)?;

    let rolling_stock_template_path = env.path_to_prejoined_zusi_path(&train_config.rolling_stock.path).map_err(|error| (&train_config.rolling_stock.path, error))?;
    let rolling_stock_template = read_zug(rolling_stock_template_path.full_path())?;

    let zug = Zug::builder()
        .gattung(train_config.gattung)
        .nummer(train_config.nummer)
        .fahrplan_datei(fahrplan_datei)
        .fahrzeug_varianten(rolling_stock_template.value.fahrzeug_varianten)
        .mindest_bremshundertstel(rolling_stock_template.value.mindest_bremshundertstel)
        .build();

    // TODO: apply train config

    Ok(
        TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(zug)
            .build()
    )
}