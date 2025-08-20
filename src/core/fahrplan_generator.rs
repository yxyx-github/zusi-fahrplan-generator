mod generate_fahrplan_error;

use crate::core::fahrplan_generator::generate_fahrplan_error::GenerateFahrplanError;
use crate::input::fahrplan_config::FahrplanConfig;
use crate::input::ZusiEnvironment;
use std::path::{Path, PathBuf};
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::{Zusi, ZusiValue};

pub fn generate_fahrplan(env: ZusiEnvironment, config: FahrplanConfig) -> Result<(), GenerateFahrplanError> {
    let mut zusi = read_fahrplan(config.generate_from)?;
    let fahrplan = match zusi {
        Zusi { value: ZusiValue::Fahrplan(ref mut fahrplan), .. } => fahrplan,
        _ => unreachable!(), // already checked inside read_fahrplan
    };

    // TODO: modify fahrplan

    zusi.to_xml_file_by_path(&config.generate_at).map_err(|error| (config.generate_at, error))?;

    todo!()
}

fn read_fahrplan<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<Zusi, GenerateFahrplanError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Fahrplan(_), .. }) => {
            Ok(zusi)
        }
        Ok(_) => Err(GenerateFahrplanError::FileTypeError {
            path: path.into(),
            expected: DateiTyp::Fahrplan,
        }),
        Err(error) => Err((path, error).into()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_generate_fahrplan() {

    }
}