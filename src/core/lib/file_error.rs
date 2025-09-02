use serde_helpers::xml::{ReadXMLFileError, WriteXMLFileError};
use std::path::PathBuf;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPathError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileError {
    pub path: PathBuf,
    pub kind: FileErrorKind,
}

impl<P: Into<PathBuf>> From<(P, FileErrorKind)> for FileError {
    fn from((path, error): (P, FileErrorKind)) -> Self {
        FileError {
            path: path.into(),
            kind: error,
        }
    }
}

impl<P: Into<PathBuf>> From<(P, ReadXMLFileError)> for FileError {
    fn from((path, error): (P, ReadXMLFileError)) -> Self {
        match error {
            ReadXMLFileError::IOError(error) => FileError {
                path: path.into(),
                kind: FileErrorKind::IOError { error: format!("{error}") },
            },
            ReadXMLFileError::DeError(error) => FileError {
                path: path.into(),
                kind: FileErrorKind::FormatError { error: format!("{error}") },
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, WriteXMLFileError)> for FileError {
    fn from((path, error): (P, WriteXMLFileError)) -> Self {
        match error {
            WriteXMLFileError::IOError(error) => FileError {
                path: path.into(),
                kind: FileErrorKind::IOError { error: format!("{error}") },
            },
            WriteXMLFileError::SeError(error) => FileError {
                path: path.into(),
                kind: FileErrorKind::FormatError { error: format!("{error}") },
            },
        }
    }
}

impl<P: Into<PathBuf>> From<(P, ZusiPathError)> for FileError {
    fn from((path, error): (P, ZusiPathError)) -> Self {
        FileError {
            path: path.into(),
            kind: FileErrorKind::InvalidPath { error, },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileErrorKind {
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

impl From<ReadXMLFileError> for FileErrorKind {
    fn from(error: ReadXMLFileError) -> Self {
        match error {
            ReadXMLFileError::IOError(error) => FileErrorKind::IOError { error: format!("{error}") },
            ReadXMLFileError::DeError(error) => FileErrorKind::FormatError { error: format!("{error}") },
        }
    }
}

impl From<WriteXMLFileError> for FileErrorKind {
    fn from(error: WriteXMLFileError) -> Self {
        match error {
            WriteXMLFileError::IOError(error) => FileErrorKind::IOError { error: format!("{error}") },
            WriteXMLFileError::SeError(error) => FileErrorKind::FormatError { error: format!("{error}") },
        }
    }
}

impl From<ZusiPathError> for FileErrorKind {
    fn from(error: ZusiPathError) -> Self {
        FileErrorKind::InvalidPath { error }
    }
}