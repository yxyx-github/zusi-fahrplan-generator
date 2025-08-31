mod file_error;
mod generate_zug;
mod helpers;

use crate::core::fahrplan_generator::file_error::FileError;
use crate::core::fahrplan_generator::generate_zug::{generate_zug, GenerateZugError};
use crate::core::fahrplan_generator::helpers::{datei_from_zusi_path, generate_zug_path, read_fahrplan};
use crate::core::fahrplan_generator::GenerateFahrplanError::ReadFahrplanTemplateError;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::FahrplanConfig;
use serde_helpers::xml::ToXML;
use zusi_xml_lib::xml::zusi::fahrplan::zug_datei_eintrag::ZugDateiEintrag;
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateFahrplanError {
    ReadFahrplanTemplateError {
        error: FileError,
    },
    WriteGeneratedFahrplanError {
        error: FileError,
    },
    GenerateZugError {
        error: GenerateZugError,
    },
    AttachZugError {
        error: FileError,
    },
}

impl From<GenerateZugError> for GenerateFahrplanError {
    fn from(error: GenerateZugError) -> Self {
        GenerateFahrplanError::GenerateZugError { error }
    }
}

pub fn generate_fahrplan(env: &ZusiEnvironment, config: FahrplanConfig) -> Result<(), GenerateFahrplanError> {
    let generate_from = env.path_to_prejoined_zusi_path(&config.generate_from)
        .map_err(|error| GenerateFahrplanError::ReadFahrplanTemplateError { error: (&config.generate_from, error).into() })?;
    let generate_at = env.path_to_prejoined_zusi_path(&config.generate_at)
        .map_err(|error| GenerateFahrplanError::WriteGeneratedFahrplanError { error: (&config.generate_at, error).into() })?;

    let mut fahrplan = read_fahrplan(generate_from.full_path())
        .map_err(|error| ReadFahrplanTemplateError { error })?;

    let zuege = config.trains
        .into_iter()
        .map(|train| generate_zug(env, &generate_at, train))
        .collect::<Result<Vec<_>, _>>()?;
    zuege
        .into_iter()
        .try_for_each(|zug| attach_zug(&mut fahrplan, zug, &generate_at))?;

    let fahrplan: Zusi = fahrplan.into();
    fahrplan.to_xml_file_by_path(generate_at.full_path())
        .map_err(|error| GenerateFahrplanError::WriteGeneratedFahrplanError { error: (generate_at.full_path(), error).into() })?;

    Ok(())
}

fn attach_zug(fahrplan: &mut TypedZusi<Fahrplan>, zug: TypedZusi<Zug>, fahrplan_path: &PrejoinedZusiPath) -> Result<(), GenerateFahrplanError> {
    let zug_path = generate_zug_path(&zug, fahrplan_path);
    let zug: Zusi = zug.into();
    fahrplan.value.zug_dateien.push(
        ZugDateiEintrag::builder()
            .datei(
                datei_from_zusi_path(zug_path.zusi_path(), false)
                .map_err(|error| GenerateFahrplanError::AttachZugError { error: (zug_path.zusi_path().get(), error).into() })?
            )
            .build()
    );
    zug.to_xml_file_by_path(fahrplan_path.full_path())
        .map_err(|error| GenerateFahrplanError::AttachZugError { error: (fahrplan_path.full_path(), error).into() })?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_generate_fahrplan() {

    }
}