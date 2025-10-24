mod utils;

use crate::utils::tmp_dir_helper::TmpDirHelper;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use serde_helpers::xml::test_utils::read_xml_file;
use std::process::Command;

#[test]
fn test_generate_fahrplan() {
    let tmp_dir = TmpDirHelper::from("./tests/generate_fahrplan/input");

    let config_path = tmp_dir.path().join("data_dir/dev/config.xml");

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    cmd.arg("generate-fahrplan").arg("-c").arg(&config_path)
        .assert()
        .stdout(predicates::str::is_match(concat!(
            r#"^Generate Fahrplan using config file at "/[a-zA-Z0-9\./-_]+/data_dir/dev/config.xml"\r?\n"#,
            r#"Zusi data dir: "/[a-zA-Z0-9\./-_]+/data_dir"\r?\n"#,
            r#"Config dir: "/[a-zA-Z0-9\./-_]+/data_dir/dev"(\r\n|\n)*$"#,
        )).unwrap())
        .stderr("")
        .success();

    tmp_dir.assert_with("./tests/generate_fahrplan/expected", true, |actual, expected| {
        assert_eq!(read_xml_file(actual), read_xml_file(expected));
    });
}