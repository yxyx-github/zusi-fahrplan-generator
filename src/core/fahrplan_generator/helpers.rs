use crate::core::fahrplan_generator::error::{GenerateFahrplanError, InvalidPathCause};
use std::path::{Path, PathBuf};
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPath;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi, ZusiValue};

pub fn read_fahrplan<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Fahrplan>, GenerateFahrplanError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Fahrplan(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err(GenerateFahrplanError::FileTypeError {
            path: path.into(),
            expected: DateiTyp::Fahrplan,
        }),
        Err(error) => Err((path, error).into()),
    }
}

pub fn read_zug<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Zug>, GenerateFahrplanError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Zug(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err(GenerateFahrplanError::FileTypeError {
            path: path.into(),
            expected: DateiTyp::Zug,
        }),
        Err(error) => Err((path, error).into()),
    }
}

pub fn datei_from_path<P: AsRef<Path> + Into<PathBuf>>(path: P, nur_info: bool) -> Result<Datei, GenerateFahrplanError> {
    match path.as_ref().to_str() {
        None => Err(GenerateFahrplanError::InvalidPath {
            path: path.into(),
            cause: InvalidPathCause::ConversionToStringFailed,
        }),
        Some(path_str) => Ok(Datei::builder().dateiname(path_str.into()).nur_info(nur_info).build()),
    }
}

pub fn datei_from_zusi_path<P: AsRef<ZusiPath> + Into<ZusiPath>>(path: P, nur_info: bool) -> Result<Datei, GenerateFahrplanError> {
    datei_from_path(path.as_ref().get(), nur_info)
}

pub fn generate_zug_path(zug: &TypedZusi<Zug>, fahrplan_path: &PrejoinedZusiPath) -> PrejoinedZusiPath {
    fahrplan_path.join_to_zusi_path(format!("{}{}.trn", zug.value.gattung, zug.value.nummer))
}

#[cfg(test)]
mod tests {

}