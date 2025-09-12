mod utils;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use glob::glob;
use serde_helpers::xml::test_utils::read_xml_file;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

const TRN_PATH: &str = "./tests/generate_schedule/input/trn.xml";

const EXPECTED_SCHEDULE_PATH: &str = "./tests/generate_schedule/expected/schedule.xml";

#[test]
fn test_generate_schedule() {
    let tmp_dir = tempdir().unwrap();

    let trn_path = tmp_dir.path().join("some/where/RB1.trn");
    fs::create_dir_all(&trn_path.parent().unwrap()).unwrap();
    fs::write(&trn_path, fs::read_to_string(TRN_PATH).unwrap()).unwrap();

    let schedule_path = tmp_dir.path().join("put/schedule/here/schedule.xml");

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    cmd.arg("schedule").arg("generate").arg("-t").arg(&trn_path).arg("-s").arg(&schedule_path)
        .assert()
        .stdout("")
        .stderr("")
        .success();
    
    assert_eq!(read_xml_file(&schedule_path), read_xml_file(EXPECTED_SCHEDULE_PATH));

    assert_eq!(fs::read_to_string(&trn_path).unwrap(), fs::read_to_string(TRN_PATH).unwrap());

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
        trn_path,
    ]);
}