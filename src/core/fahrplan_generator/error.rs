use std::path::PathBuf;
use serde_helpers::xml::{ReadXMLFileError, WriteXMLFileError};
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::InvalidBasePath;
use crate::core::schedule::apply::ApplyScheduleError;
use crate::input::fahrplan_config::RoutePartSource;

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
        cause: InvalidPathCause,
    },
    NoRouteParts {
        zug_nummer: String,
    },
    EmptyRoutePart {
        source: RoutePartSource,
    },
    InvalidSchedule {
        zug_nummer: String,
        path: PathBuf,
        error: ApplyScheduleError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidPathCause {
    ConversionToStringFailed,
    InvalidBasePath,
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

impl<P: Into<PathBuf>> From<(P, InvalidBasePath)> for GenerateFahrplanError {
    fn from((path, _): (P, InvalidBasePath)) -> Self {
        GenerateFahrplanError::InvalidPath {
            path: path.into(),
            cause: InvalidPathCause::InvalidBasePath,
        }
    }
}