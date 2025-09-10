use std::io;
use std::io::Error;
use serde_helpers::xml::{ReadXMLFileError, WriteXMLFileError};
use std::path::PathBuf;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::info::DateiTyp;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPathError;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error(r#"A file error occoured for "{}": {kind}"#, path.display())]
pub struct FileError {
    pub path: PathBuf,

    #[source]
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

impl<P: Into<PathBuf>> From<(P, io::Error)> for FileError {
    fn from((path, error): (P, io::Error)) -> Self {
        FileError {
            path: path.into(),
            kind: FileErrorKind::IOError { error: format!("{error}") },
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

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum FileErrorKind {
    #[error("An IO error occoured: {error}")]
    IOError {
        error: String,
    },

    #[error("The format of the file content is invalid: {error}")]
    FormatError {
        error: String,
    },

    #[error("The format of the file content is invalid: {expected:?}")]
    WrongType {
        expected: DateiTyp,
    },

    #[error("The path is invalid: {error}")]
    InvalidPath {
        error: ZusiPathError,
    },

    #[error("The path must point to a file.")]
    MustBeFile,

    #[error("The path must have a parent directory.")]
    MustHaveParent,
}

impl From<io::Error> for FileErrorKind {
    fn from(error: Error) -> Self {
        FileErrorKind::IOError { error: format!("{error}") }
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