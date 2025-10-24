use glob::glob;
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::{tempdir, TempDir};

pub struct TmpDirHelper {
    _tmp_dir: TempDir,
    tmp_dir_path: PathBuf,
    input_dir: PathBuf,
}

// TODO: don't use unwrap
impl TmpDirHelper {
    pub fn from<P: AsRef<Path>>(input_dir: P) -> Self {
        let tmp_dir = tempdir().unwrap();
        let tmp_dir_path = tmp_dir.path().canonicalize().unwrap();
        let input_dir = input_dir.as_ref().canonicalize().unwrap();
        let mut helper = Self {
            _tmp_dir: tmp_dir,
            tmp_dir_path,
            input_dir,
        };
        helper.write_input();
        helper
    }

    pub fn path(&self) -> &PathBuf {
        &self.tmp_dir_path
    }

    fn write_input(&mut self) {
        self.zip_corresponding_tmp_dir_path(all_file_paths(&self.input_dir), &self.input_dir).into_iter().for_each(|(input_path, tmp_dir_path)| {
            if let Some(parent) = input_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            if let Some(parent) = tmp_dir_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(input_path, tmp_dir_path).unwrap();
        })
    }

    pub fn assert<P: AsRef<Path>, F: Fn(PathBuf, PathBuf)>(&self, expected_output_dir: P, with_input: bool) {
        self.assert_with(expected_output_dir, with_input, |actual, expected| {
            assert_eq!(fs::read_to_string(actual).unwrap(), fs::read_to_string(expected).unwrap());
        });
    }

    pub fn assert_with<P: AsRef<Path>, F: Fn(PathBuf, PathBuf)>(&self, expected_output_dir: P, with_input: bool, assert_fn: F) {
        let expected_output_dir = expected_output_dir.as_ref().canonicalize().unwrap();
        let mut assertion_paths = self.zip_corresponding_tmp_dir_path(all_file_paths(&expected_output_dir), &expected_output_dir);
        if with_input {
            let input_assertion_paths = self
                .zip_corresponding_tmp_dir_path(all_file_paths(&self.input_dir), &self.input_dir)
                .into_iter()
                .filter(|(_, new_tmp_dir_path)|
                    !assertion_paths.iter().any(|(_, existing_tmp_dir_path)|
                        existing_tmp_dir_path == new_tmp_dir_path
                    )
                )
                .collect::<Vec<_>>();
            assertion_paths.extend(input_assertion_paths);
        }
        assertion_paths.sort_by(|(_, tmp_dir_path_1), (_, tmp_dir_path_2)| tmp_dir_path_1.cmp(tmp_dir_path_2));

        assert_eq!(
            assertion_paths.iter().map(|(_, tmp_dir_path)| tmp_dir_path.clone()).collect::<Vec<_>>(),
            all_file_paths(&self.tmp_dir_path),
        );

        assertion_paths.into_iter().for_each(|(path_expected, path_actual)| assert_fn(path_actual, path_expected));
    }

    fn zip_corresponding_tmp_dir_path<P: AsRef<Path>>(&self, paths: Vec<PathBuf>, original_dir: P) -> Vec<(PathBuf, PathBuf)> {
        paths.into_iter().map(|path| {
            let inner_path = path.strip_prefix(original_dir.as_ref()).unwrap();
            let new_path = self.tmp_dir_path.join(inner_path);
            (path, new_path)
        }).collect()
    }
}

// TODO: don't use unwrap
fn all_file_paths<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    glob(
        dir.as_ref().join("**/*.*").to_str().unwrap()
    )
        .unwrap()
        .into_iter()
        .map(|path|
            path.unwrap()
        )
        .collect()
}