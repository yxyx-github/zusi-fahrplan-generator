mod error;
mod generate_train;
mod helpers;

use serde_helpers::xml::ToXML;
use crate::core::fahrplan_generator::error::GenerateFahrplanError;
use crate::core::fahrplan_generator::generate_train::generate_zug;
use crate::core::fahrplan_generator::helpers::{datei_from_zusi_path, generate_zug_path, read_fahrplan};
use crate::input::fahrplan_config::FahrplanConfig;
use zusi_xml_lib::xml::zusi::fahrplan::zug_datei_eintrag::ZugDateiEintrag;
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi};
use crate::input::environment::zusi_environment::ZusiEnvironment;

pub fn generate_fahrplan(env: &ZusiEnvironment, config: FahrplanConfig) -> Result<(), GenerateFahrplanError> {
    let generate_from = env.path_to_prejoined_zusi_path(&config.generate_from).map_err(|error| (&config.generate_from, error))?;
    let generate_at = env.path_to_prejoined_zusi_path(&config.generate_at).map_err(|error| (&config.generate_at, error))?;

    let mut fahrplan = read_fahrplan(generate_from.full_path())?;

    let zuege: Result<Vec<_>, _> = config.trains.into_iter().map(|train| generate_zug(env, &generate_at, train)).collect();
    let zuege = zuege?;
    zuege.into_iter().try_for_each(|zug| attach_zug(&mut fahrplan, zug, &generate_at))?;

    let fahrplan: Zusi = fahrplan.into();
    fahrplan.to_xml_file_by_path(generate_at.full_path()).map_err(|error| (generate_at.full_path(), error))?;

    Ok(())
}

fn attach_zug(fahrplan: &mut TypedZusi<Fahrplan>, zug: TypedZusi<Zug>, fahrplan_path: &PrejoinedZusiPath) -> Result<(), GenerateFahrplanError> {
    let zug_path = generate_zug_path(&zug, fahrplan_path);
    let zug: Zusi = zug.into();
    zug.to_xml_file_by_path(fahrplan_path.full_path()).map_err(|error| (fahrplan_path.full_path().with_extension(""), error))?;
    fahrplan.value.zug_dateien.push(ZugDateiEintrag::builder().datei(datei_from_zusi_path(zug_path.zusi_path(), false)?).build());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_generate_fahrplan() {

    }
}