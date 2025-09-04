use std::fmt::{Display, Formatter};
use crate::core::lib::file_error::{FileError, FileErrorKind};
use crate::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
use std::fs;
use std::path::{absolute, Path, PathBuf};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZusiEnvironment {
    /// Path to own data directory root
    pub data_dir: PathBuf,

    /// Path to directory that contains the configuration file used as input
    pub config_dir: PathBuf,
}

impl ZusiEnvironment {
    pub fn from_zusi_environment_config<T>(config: ZusiEnvironmentConfig<T>, config_path: PathBuf) -> Result<(ZusiEnvironment, T), FileError> {
        let config_path = absolute(&config_path)
            .map_err(|error| FileError::from((config_path, error)))?;
        let config_dir = config_path.parent().ok_or(
            FileError {
                path: config_path.clone(),
                kind: FileErrorKind::MustHaveParent,
            }
        )?;
        let config_dir = fs::canonicalize(&config_dir)
            .map_err(|error| FileError::from((config_dir, error)))?;

        let data_dir = if config.data_dir.is_absolute() {
            config.data_dir
        } else {
            config_dir.join(config.data_dir)
        };
        let data_dir = fs::canonicalize(&data_dir)
            .map_err(|error| FileError::from((data_dir, error)))?;

        Ok((
            ZusiEnvironment {
                data_dir,
                config_dir: config_dir.into(),
            },
            config.value,
        ))
    }

    pub fn path_to_prejoined_zusi_path<P: AsRef<Path> + Into<PathBuf>>(&self, path: P) -> Result<PrejoinedZusiPath, FileError> {
        Ok(
            if path.as_ref().is_absolute() {
                PrejoinedZusiPath::new(
                    &self.data_dir,
                    ZusiPath::new(path.into().strip_prefix("/").unwrap())
                        .map_err(|error| FileError::from((&self.data_dir, error)))?,
                )
            } else {
                let root_path = self.config_dir.join(&path);
                let zusi_path = ZusiPath::new_using_data_dir(&root_path, &self.data_dir)
                    .map_err(|error| FileError::from((root_path, error)))?;
                PrejoinedZusiPath::new(&self.data_dir, zusi_path)
            }
        )
    }
}

impl Display for ZusiEnvironment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Zusi data dir: {:?}", self.data_dir)?;
        writeln!(f, "Config dir: {:?}", self.config_dir)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use super::*;
    use crate::core::lib::file_error::FileErrorKind;
    use zusi_xml_lib::xml::zusi::lib::path::zusi_path::ZusiPathError;

    fn dir_path<P: AsRef<Path>>(tmp_dir: &TempDir, path: P) -> PathBuf {
        let path = tmp_dir.path().join(path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn file_path<P: AsRef<Path>>(tmp_dir: &TempDir, path: P) -> PathBuf {
        let path = tmp_dir.path().join(path);

        fs::create_dir_all(&path.parent().unwrap()).unwrap();
        fs::write(&path, "").unwrap();
        path
    }

    #[test]
    fn test_from_zusi_environment_config_config_dir_equals_data_dir() {
        let tmp_dir = tempdir().unwrap();

        let path_to_dir = dir_path(&tmp_dir, "path/to/dir");
        let path_to_config = file_path(&tmp_dir, "path/to/dir/config.xml");

        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: path_to_dir.clone(),
                value: (),
            }.into_zusi_environment(path_to_config).unwrap(),
            (ZusiEnvironment {
                data_dir: path_to_dir.clone(),
                config_dir: path_to_dir,
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_config_dir_inside_data_dir() {
        let tmp_dir = tempdir().unwrap();

        let path_to_data_dir = dir_path(&tmp_dir, "path/to/data_dir");
        let path_to_config_dir = dir_path(&tmp_dir, "path/to/data_dir/and/then/config_dir");
        let path_to_config = file_path(&tmp_dir, "path/to/data_dir/and/then/config_dir/config.xml");

        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: path_to_data_dir.clone(),
                value: (),
            }.into_zusi_environment(path_to_config).unwrap(),
            (ZusiEnvironment {
                data_dir: path_to_data_dir,
                config_dir: path_to_config_dir,
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_config_dir_inside_data_dir_relative_data_dir() {
        let tmp_dir = tempdir().unwrap();

        let input_path_to_data_dir = PathBuf::from("../../../");
        let path_to_data_dir = dir_path(&tmp_dir, "path/to/data_dir");
        let path_to_config_dir = dir_path(&tmp_dir, "path/to/data_dir/and/then/config_dir");
        let path_to_config = file_path(&tmp_dir, "path/to/data_dir/and/then/config_dir/config.xml");

        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: input_path_to_data_dir,
                value: (),
            }.into_zusi_environment(path_to_config).unwrap(),
            (ZusiEnvironment {
                data_dir: path_to_data_dir,
                config_dir: path_to_config_dir,
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_config_dir_inside_data_dir_relative_config_dir() {
        let tmp_dir = tempdir().unwrap();

        let path_to_data_dir = dir_path(&tmp_dir, "path/to/data_dir");
        let path_to_config_dir = dir_path(&tmp_dir, "path/to/data_dir/and/then/config_dir");
        let path_to_config = file_path(&tmp_dir, "path/to/data_dir/and/then/../then/config_dir/config.xml");

        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: path_to_data_dir.clone(),
                value: (),
            }.into_zusi_environment(path_to_config).unwrap(),
            (ZusiEnvironment {
                data_dir: path_to_data_dir,
                config_dir: path_to_config_dir,
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_data_dir_inside_config_dir() {
        let tmp_dir = tempdir().unwrap();

        let path_to_data_dir = dir_path(&tmp_dir, "path/to/config_dir/and/then/data_dir");
        let path_to_config_dir = dir_path(&tmp_dir, "path/to/config_dir");
        let path_to_config = file_path(&tmp_dir, "path/to/config_dir/config.xml");

        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: path_to_data_dir.clone(),
                value: (),
            }.into_zusi_environment(path_to_config).unwrap(),
            (ZusiEnvironment {
                data_dir: path_to_data_dir,
                config_dir: path_to_config_dir,
            }, ()),
        );
    }

    #[test]
    fn test_path_to_prejoined_zusi_path_config_dir_equals_data_dir() {
        let env = ZusiEnvironment {
            data_dir: "/path/to/dir".into(),
            config_dir: "/path/to/dir".into(),
        };

        let prejoined_zusi_path = env.path_to_prejoined_zusi_path("to/any/file").unwrap();

        assert_eq!(prejoined_zusi_path.data_dir().to_str().unwrap(), "/path/to/dir");
        assert_eq!(prejoined_zusi_path.zusi_path().get().to_str().unwrap(), "to/any/file");
        assert_eq!(prejoined_zusi_path.full_path().to_str().unwrap(), "/path/to/dir/to/any/file");
    }

    #[test]
    fn test_path_to_prejoined_zusi_path_config_dir_inside_data_dir() {
        let env = ZusiEnvironment {
            data_dir: "/path/to/data_dir".into(),
            config_dir: "/path/to/data_dir/and/then/config_dir".into(),
        };

        let prejoined_zusi_path = env.path_to_prejoined_zusi_path("to/any/file").unwrap();

        assert_eq!(prejoined_zusi_path.data_dir().to_str().unwrap(), "/path/to/data_dir");
        assert_eq!(prejoined_zusi_path.zusi_path().get().to_str().unwrap(), "and/then/config_dir/to/any/file");
        assert_eq!(prejoined_zusi_path.full_path().to_str().unwrap(), "/path/to/data_dir/and/then/config_dir/to/any/file");
    }

    #[test]
    fn test_path_to_prejoined_zusi_path_data_dir_inside_config_dir() {
        let env = ZusiEnvironment {
            data_dir: "/path/to/config_dir/and/then/data_dir".into(),
            config_dir: "/path/to/config_dir".into(),
        };

        let prejoined_zusi_path = env.path_to_prejoined_zusi_path("/to/any/file").unwrap();

        assert_eq!(prejoined_zusi_path.data_dir().to_str().unwrap(), "/path/to/config_dir/and/then/data_dir");
        assert_eq!(prejoined_zusi_path.zusi_path().get().to_str().unwrap(), "to/any/file");
        assert_eq!(prejoined_zusi_path.full_path().to_str().unwrap(), "/path/to/config_dir/and/then/data_dir/to/any/file");
    }

    #[test]
    fn test_path_to_prejoined_zusi_path_with_invalid_base_path() {
        let env = ZusiEnvironment {
            data_dir: "/path/to/data_dir".into(),
            config_dir: "/path/to/other/and/then/config_dir".into(),
        };

        assert_eq!(
            env.path_to_prejoined_zusi_path("to/any/file").unwrap_err(),
            FileError {
                path: "/path/to/other/and/then/config_dir/to/any/file".into(),
                kind: FileErrorKind::InvalidPath { error: ZusiPathError::PathDoesNotContainDataDir },
            },
        );
    }

    #[test]
    fn test_path_to_prejoined_zusi_path_with_invalid_base_path_data_dir_inside_config_dir() {
        let env = ZusiEnvironment {
            data_dir: "/path/to/config_dir/and/then/data_dir".into(),
            config_dir: "/path/to/config_dir".into(),
        };

        assert_eq!(
            env.path_to_prejoined_zusi_path("to/any/file").unwrap_err(),
            FileError {
                path: "/path/to/config_dir/to/any/file".into(),
                kind: FileErrorKind::InvalidPath { error: ZusiPathError::PathDoesNotContainDataDir },
            },
        );
    }
}