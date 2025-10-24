mod utils;

use crate::utils::tmp_dir_helper::TmpDirHelper;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_helpers::xml::test_utils::read_xml_file;
use std::process::Command;

#[test]
fn test_generate_schedule() {
    let tmp_dir = TmpDirHelper::from("./tests/generate_schedule/input");

    let trn_path = tmp_dir.path().join("some/where/RB1.trn");
    let schedule_path = tmp_dir.path().join("put/schedule/here/schedule.xml");

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    cmd.arg("schedule").arg("generate").arg("-t").arg(&trn_path).arg("-s").arg(&schedule_path)
        .assert()
        .stdout("")
        .stderr("")
        .success();

    tmp_dir.assert_with("./tests/generate_schedule/expected", true, |actual, expected| {
        assert_eq!(read_xml_file(actual), read_xml_file(expected));
    });
}