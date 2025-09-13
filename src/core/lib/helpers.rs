use crate::core::lib::file_error::{FileError, FileErrorKind};
use serde_helpers::xml::FromXML;
use std::path::{Component, Path, PathBuf};
use serde_helpers::default::IsDefault;
use time::Duration;
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::datei::Datei;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPath;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi, ZusiValue};

pub fn read_fahrplan<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Fahrplan>, FileError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Fahrplan(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err((path, FileErrorKind::WrongType { expected: DateiTyp::Fahrplan }).into()),
        Err(error) => Err((path, error).into()),
    }
}

pub fn read_zug<P: AsRef<Path> + Into<PathBuf>>(path: P) -> Result<TypedZusi<Zug>, FileError> {
    match Zusi::from_xml_file_by_path(path.as_ref()) {
        Ok(zusi @ Zusi { value: ZusiValue::Zug(_), .. }) => {
            Ok(zusi.try_into().unwrap())
        }
        Ok(_) => Err((path, FileErrorKind::WrongType { expected: DateiTyp::Zug }).into()),
        Err(error) => Err((path, error).into()),
    }
}

pub fn datei_from_path<P: Into<PathBuf>>(path: P, nur_info: bool) -> Result<Datei, FileError> {
    let path = path.into();
    let zusi_path = ZusiPath::try_from(path.clone()) // TODO: do not clone
        .map_err(|error| FileError::from((path, error)))?;
    datei_from_zusi_path(zusi_path, nur_info)
}

pub fn datei_from_prejoined_zusi_path<P: AsRef<PrejoinedZusiPath> + Into<PrejoinedZusiPath>>(path: P, nur_info: bool) -> Result<Datei, FileError> {
    datei_from_zusi_path(path.as_ref().zusi_path(), nur_info)
}

pub fn datei_from_zusi_path<P: AsRef<ZusiPath> + Into<ZusiPath>>(path: P, nur_info: bool) -> Result<Datei, FileError> {
    Ok(Datei::builder().dateiname(path.into()).nur_info(nur_info).build())
}

pub fn generate_zug_path(zug: &TypedZusi<Zug>, fahrplan_path: &PrejoinedZusiPath) -> PrejoinedZusiPath {
    PrejoinedZusiPath::new(
        fahrplan_path.data_dir(), fahrplan_path
        .zusi_path()
        .get()
        .with_extension("")
        .join(format!("{}{}.trn", zug.value.gattung, zug.value.nummer)).try_into().unwrap()
    )
}

pub fn path_to_relative<P: AsRef<Path> + Into<PathBuf>>(path: P) -> PathBuf {
    path.as_ref().components().filter(|component| !matches!(component, Component::Prefix(_) | Component::RootDir)).collect()
}

pub fn delay_fahrplan_eintraege(eintraege: &mut Vec<FahrplanEintrag>, delay: Duration) {
    eintraege.iter_mut().for_each(|eintrag| {
        eintrag.ankunft = eintrag.ankunft.map(|time| time + delay);
        eintrag.abfahrt = eintrag.abfahrt.map(|time| time + delay);
    })
}

pub fn override_default<T: IsDefault>(a: &mut T, b: T) {
    if a.is_default() {
        *a = b;
    }
}

pub fn override_with_non_default<T: IsDefault>(a: &mut T, b: T) {
    if !b.is_default() {
        *a = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zusi_xml_lib::xml::zusi::info::Info;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;

    #[test]
    fn test_generate_zug_path() {
        let zug = TypedZusi::<Zug>::builder()
            .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
            .value(
                Zug::builder()
                    .fahrplan_datei(Datei::builder().build())
                    .gattung("RE".into())
                    .nummer("123".into())
                    .fahrzeug_varianten(FahrzeugVarianten::builder().build())
                    .build()
            )
            .build();

        let fahrplan_dir = PrejoinedZusiPath::new("to/data_dir", ZusiPath::new("the/fahrplan/dir").unwrap());
        let fahrplan_path = PrejoinedZusiPath::new("to/data_dir", ZusiPath::new("the/fahrplan/dir.fpn").unwrap());

        let expected = PrejoinedZusiPath::new("to/data_dir", ZusiPath::new("the/fahrplan/dir/RE123.trn").unwrap());

        assert_eq!(generate_zug_path(&zug, &fahrplan_dir), expected);
        assert_eq!(generate_zug_path(&zug, &fahrplan_path), expected);
    }

    #[test]
    fn test_path_to_relative() {
        assert_eq!(path_to_relative("/a/b/c.d"), PathBuf::from("a/b/c.d"));
        assert_eq!(path_to_relative("a/b/c.d"), PathBuf::from("a/b/c.d"));
        #[cfg(target_os = "windows")]
        assert_eq!(path_to_relative(r"C:\a\b\c.d"), PathBuf::from(r"a\b\c.d"));
    }
}