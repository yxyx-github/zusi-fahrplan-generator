use std::path::PathBuf;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::{ReadZusiXMLFileError, WriteZusiXMLFileError};
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::InvalidBasePath;

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
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidPathCause {
    ConversionToStringFailed,
    InvalidBasePath,
}

impl<P: Into<PathBuf>> From<(P, ReadZusiXMLFileError)> for GenerateFahrplanError {
    fn from((path, error): (P, ReadZusiXMLFileError)) -> Self {
        match error {
            ReadZusiXMLFileError::IOError(error) => GenerateFahrplanError::ReadFileError {
                path: path.into(),
                error: error.to_string(),
            },
            ReadZusiXMLFileError::DeError(error) => GenerateFahrplanError::ReadFileError {
                path: path.into(),
                error: error.to_string(),
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, WriteZusiXMLFileError)> for GenerateFahrplanError {
    fn from((path, error): (P, WriteZusiXMLFileError)) -> Self {
        match error {
            WriteZusiXMLFileError::IOError(error) => GenerateFahrplanError::WriteFileError {
                path: path.into(),
                error: error.to_string(),
            },
            WriteZusiXMLFileError::SeError(error) => GenerateFahrplanError::WriteFileError {
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