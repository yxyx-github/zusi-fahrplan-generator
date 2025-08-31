use serde_helpers::xml::{ReadXMLFileError, WriteXMLFileError};
use std::path::PathBuf;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPathError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileError {
    pub path: PathBuf,
    pub cause: FileErrorCause,
}

impl<P: Into<PathBuf>> From<(P, FileErrorCause)> for FileError {
    fn from((path, error): (P, FileErrorCause)) -> Self {
        FileError {
            path: path.into(),
            cause: error,
        }
    }
}

impl<P: Into<PathBuf>> From<(P, ReadXMLFileError)> for FileError {
    fn from((path, error): (P, ReadXMLFileError)) -> Self {
        match error {
            ReadXMLFileError::IOError(error) => FileError {
                path: path.into(),
                cause: FileErrorCause::IOError { error: format!("{error}") },
            },
            ReadXMLFileError::DeError(error) => FileError {
                path: path.into(),
                cause: FileErrorCause::FormatError { error: format!("{error}") },
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, WriteXMLFileError)> for FileError {
    fn from((path, error): (P, WriteXMLFileError)) -> Self {
        match error {
            WriteXMLFileError::IOError(error) => FileError {
                path: path.into(),
                cause: FileErrorCause::IOError { error: format!("{error}") },
            },
            WriteXMLFileError::SeError(error) => FileError {
                path: path.into(),
                cause: FileErrorCause::FormatError { error: format!("{error}") },
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, ZusiPathError)> for FileError {
    fn from((path, error): (P, ZusiPathError)) -> Self {
        FileError {
            path: path.into(),
            cause: FileErrorCause::InvalidPath { error, },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileErrorCause {
    IOError {
        error: String,
    },
    FormatError {
        error: String,
    },
    WrongType {
        expected: DateiTyp,
    },
    InvalidPath {
        error: ZusiPathError,
    }
}

impl From<ReadXMLFileError> for FileErrorCause {
    fn from(error: ReadXMLFileError) -> Self {
        match error {
            ReadXMLFileError::IOError(error) => FileErrorCause::IOError { error: format!("{error}") },
            ReadXMLFileError::DeError(error) => FileErrorCause::FormatError { error: format!("{error}") },
        }
    }
}

impl From<WriteXMLFileError> for FileErrorCause {
    fn from(error: WriteXMLFileError) -> Self {
        match error {
            WriteXMLFileError::IOError(error) => FileErrorCause::IOError { error: format!("{error}") },
            WriteXMLFileError::SeError(error) => FileErrorCause::FormatError { error: format!("{error}") },
        }
    }
}

impl From<ZusiPathError> for FileErrorCause {
    fn from(error: ZusiPathError) -> Self {
        FileErrorCause::InvalidPath { error }
    }
}