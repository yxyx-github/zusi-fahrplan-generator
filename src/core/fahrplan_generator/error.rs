use crate::core::schedules::apply::ApplyScheduleError;
use crate::input::fahrplan_config::RoutePartSource;
use serde_helpers::xml::{ReadXMLFileError, WriteXMLFileError};
use std::path::PathBuf;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPathError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerateFahrplanError {
    ReadFileError {
        path: PathBuf,
        error: String,
    },
    WriteFileError {
        path: PathBuf,
        error: String,
    },
    FileTypeError {
        path: PathBuf,
        expected: DateiTyp,
    },
    InvalidPath {
        path: PathBuf,
        error: ZusiPathError,
    },
    NoRouteParts {
        zug_nummer: String,
    },
    EmptyRoutePart {
        source: RoutePartSource,
    },
    CouldNotApplySchedule {
        zug_nummer: String,
        path: PathBuf,
        error: ApplyScheduleError,
    },
    CouldNotApplyTimeFix {
        zug_nummer: String,
    },
}

impl<P: Into<PathBuf>> From<(P, ReadXMLFileError)> for GenerateFahrplanError {
    fn from((path, error): (P, ReadXMLFileError)) -> Self {
        match error {
            ReadXMLFileError::IOError(error) => GenerateFahrplanError::ReadFileError {
                path: path.into(),
                error: error.to_string(),
            },
            ReadXMLFileError::DeError(error) => GenerateFahrplanError::ReadFileError {
                path: path.into(),
                error: error.to_string(),
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, WriteXMLFileError)> for GenerateFahrplanError {
    fn from((path, error): (P, WriteXMLFileError)) -> Self {
        match error {
            WriteXMLFileError::IOError(error) => GenerateFahrplanError::WriteFileError {
                path: path.into(),
                error: error.to_string(),
            },
            WriteXMLFileError::SeError(error) => GenerateFahrplanError::WriteFileError {
                path: path.into(),
                error: error.to_string(),
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, ZusiPathError)> for GenerateFahrplanError {
    fn from((path, error): (P, ZusiPathError)) -> Self {
        GenerateFahrplanError::InvalidPath {
            path: path.into(),
            error,
        }
    }
}