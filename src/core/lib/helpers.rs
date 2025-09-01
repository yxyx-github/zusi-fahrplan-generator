use crate::core::lib::file_error::{FileError, FileErrorCause};
use serde_helpers::xml::FromXML;
use std::path::{Path, PathBuf};
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::{ZusiPath, ZusiPathError};
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi, ZusiValue};

pub fn read_fahrplan<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Fahrplan>, FileError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Fahrplan(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err((path, FileErrorCause::WrongType { expected: DateiTyp::Fahrplan }).into()),
        Err(error) => Err((path, error).into()),
    }
}

pub fn read_zug<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Zug>, FileError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Zug(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err((path, FileErrorCause::WrongType { expected: DateiTyp::Zug }).into()),
        Err(error) => Err((path, error).into()),
    }
}

pub fn datei_from_path<P: Into<PathBuf>>(path: P, nur_info: bool) -> Result<Datei, ZusiPathError> {
    let zusi_path = path.into().try_into()?;

    Ok(Datei::builder().dateiname(zusi_path).nur_info(nur_info).build())
}

pub fn datei_from_zusi_path<P: AsRef<ZusiPath> + Into<ZusiPath>>(path: P, nur_info: bool) -> Result<Datei, ZusiPathError> {
    datei_from_path(path.as_ref().get(), nur_info)
}

pub fn generate_zug_path(zug: &TypedZusi<Zug>, fahrplan_path: &PrejoinedZusiPath) -> PrejoinedZusiPath {
    fahrplan_path.join_to_zusi_path(format!("./{}{}.trn", zug.value.gattung, zug.value.nummer)).unwrap()
}

#[cfg(test)]
mod tests {

}