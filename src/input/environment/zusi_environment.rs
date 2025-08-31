use crate::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
use std::path::{Path, PathBuf};
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::lib::path::zusi_path::{ZusiPath, ZusiPathError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZusiEnvironment {
    /// Path to own data directory root
    pub data_dir: PathBuf,

    /// Path to directory that contains the configuration file used as input
    pub config_dir: PathBuf,
}

impl ZusiEnvironment {
    pub fn from_zusi_environment_config<T>(config: ZusiEnvironmentConfig<T>, config_path: PathBuf) -> (ZusiEnvironment, T) {
        (
            ZusiEnvironment {
                data_dir: config.data_dir,
                config_dir: config_path,
            },
            config.value,
        )
    }

    pub fn path_to_prejoined_zusi_path<P: AsRef<Path> + Into<PathBuf>>(&self, path: P) -> Result<PrejoinedZusiPath, ZusiPathError> {
        Ok(
            if path.as_ref().is_absolute() {
                PrejoinedZusiPath::new(&self.data_dir, ZusiPath::new(path.into().strip_prefix("/").unwrap())?)
            } else {
                let root_path = self.config_dir.join(&path);
                let zusi_path = ZusiPath::new_using_data_dir(root_path, &self.data_dir)?;
                PrejoinedZusiPath::new(&self.data_dir, zusi_path)
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_zusi_environment_config_config_dir_equals_data_dir() {
        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: "/path/to/dir".into(),
                value: (),
            }.into_zusi_environment("/path/to/dir".into()),
            (ZusiEnvironment {
                data_dir: "/path/to/dir".into(),
                config_dir: "/path/to/dir".into(),
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_config_dir_inside_data_dir() {
        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: "/path/to/data_dir".into(),
                value: (),
            }.into_zusi_environment("/path/to/data_dir/and/then/config_dir".into()),
            (ZusiEnvironment {
                data_dir: "/path/to/data_dir".into(),
                config_dir: "/path/to/data_dir/and/then/config_dir".into(),
            }, ()),
        );
    }

    #[test]
    fn test_from_zusi_environment_config_data_dir_inside_config_dir() {
        assert_eq!(
            ZusiEnvironmentConfig {
                data_dir: "/path/to/config_dir/and/then/data_dir".into(),
                value: (),
            }.into_zusi_environment("/path/to/config_dir".into()),
            (ZusiEnvironment {
                data_dir: "/path/to/config_dir/and/then/data_dir".into(),
                config_dir: "/path/to/config_dir".into(),
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
            ZusiPathError::PathDoesNotContainDataDir,
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
            ZusiPathError::PathDoesNotContainDataDir,
        );
    }
}