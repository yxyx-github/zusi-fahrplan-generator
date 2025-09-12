mod utils;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use glob::glob;
use serde_helpers::xml::test_utils::read_xml_file;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

const SCHEDULE_PATH: &str = "./tests/apply_schedule/input/schedule.xml";
const TRN_1_PATH: &str = "./tests/apply_schedule/input/trn-1.xml";
const TRN_2_PATH: &str = "./tests/apply_schedule/input/trn-2.xml";

const EXPECTED_TRN_1_PATH: &str = "./tests/apply_schedule/expected/trn-1.xml";
const EXPECTED_TRN_2_PATH: &str = "./tests/apply_schedule/expected/trn-2.xml";

#[test]
fn test_apply_schedule() {
    let tmp_dir = tempdir().unwrap();

    let schedule_path = tmp_dir.path().join("path/to/schedule.xml");
    fs::create_dir_all(&schedule_path.parent().unwrap()).unwrap();
    fs::write(&schedule_path, fs::read_to_string(SCHEDULE_PATH).unwrap()).unwrap();

    let trn_1_path = tmp_dir.path().join("some/where/RB1.trn");
    fs::create_dir_all(&trn_1_path.parent().unwrap()).unwrap();
    fs::write(&trn_1_path, fs::read_to_string(TRN_1_PATH).unwrap()).unwrap();

    let trn_2_path = tmp_dir.path().join("some/else/where/RB2.trn");
    fs::create_dir_all(&trn_2_path.parent().unwrap()).unwrap();
    fs::write(&trn_2_path, fs::read_to_string(TRN_2_PATH).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    cmd.arg("schedule").arg("apply").arg("-s").arg(&schedule_path).arg("-t").arg(&trn_1_path).arg(&trn_2_path)
        .assert()
        .stdout("")
        .stderr("")
        .success();

    assert_eq!(read_xml_file(&trn_1_path), read_xml_file(EXPECTED_TRN_1_PATH));
    assert_eq!(read_xml_file(&trn_2_path), read_xml_file(EXPECTED_TRN_2_PATH));

    assert_eq!(fs::read_to_string(&schedule_path).unwrap(), fs::read_to_string(SCHEDULE_PATH).unwrap());

    let all_file_paths: Vec<PathBuf> = glob(
        tmp_dir.path().join("**/*.*").to_str().unwrap()
    )
        .unwrap()
        .into_iter()
        .map(|path|
            path.unwrap()
        )
        .collect();

    assert_eq!(all_file_paths, vec![
        schedule_path,
        trn_2_path,
        trn_1_path,
    ]);
}