mod utils;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use glob::glob;
use serde_helpers::xml::test_utils::read_xml_file;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

const CONFIG_PATH: &str = "./tests/generate_fahrplan/input/config.xml";
const FPN_TPL_PATH: &str = "./tests/generate_fahrplan/input/my-fahrplan.xml";
const ROUTE_1_TPL_PATH: &str = "./tests/generate_fahrplan/input/route-part-1.xml";
const ROUTE_2_TPL_PATH: &str = "./tests/generate_fahrplan/input/route-part-2.xml";
const META_DATA_TPL_PATH: &str = "./tests/generate_fahrplan/input/meta-data.xml";
const ROLLING_STOCK_A_TPL_PATH: &str = "./tests/generate_fahrplan/input/rolling-stock-a.xml";
const ROLLING_STOCK_B_TPL_PATH: &str = "./tests/generate_fahrplan/input/rolling-stock-b.xml";
const ROUTE_1_2_SCHEDULE_PATH: &str = "./tests/generate_fahrplan/input/route-part-1-2.schedule.xml";

const EXPECTED_FPN_PATH: &str = "./tests/generate_fahrplan/expected/my-fahrplan.xml";
const EXPECTED_TRN_1_PATH: &str = "./tests/generate_fahrplan/expected/trn-1.xml";
const EXPECTED_TRN_2_PATH: &str = "./tests/generate_fahrplan/expected/trn-2.xml";
const EXPECTED_TRN_3_PATH: &str = "./tests/generate_fahrplan/expected/trn-3.xml";
const EXPECTED_TRN_4_PATH: &str = "./tests/generate_fahrplan/expected/trn-4.xml";

#[test]
fn test_generate_fahrplan() {
    let tmp_dir = tempdir().unwrap();

    let config_path = tmp_dir.path().join("data_dir/dev/config.xml");
    fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, fs::read_to_string(CONFIG_PATH).unwrap()).unwrap();

    let fpn_tpl_path = tmp_dir.path().join("data_dir/dev/my-fahrplan.fpn");
    fs::create_dir_all(&fpn_tpl_path.parent().unwrap()).unwrap();
    fs::write(&fpn_tpl_path, fs::read_to_string(FPN_TPL_PATH).unwrap()).unwrap();
    fs::write(&config_path, fs::read_to_string(CONFIG_PATH).unwrap()).unwrap();

    let meta_data_tpl_path = tmp_dir.path().join("data_dir/dev/meta-data.trn");
    fs::create_dir_all(&meta_data_tpl_path.parent().unwrap()).unwrap();
    fs::write(&meta_data_tpl_path, fs::read_to_string(META_DATA_TPL_PATH).unwrap()).unwrap();

    let rolling_stock_a_tpl_path = tmp_dir.path().join("data_dir/dev/rolling-stock-a.trn");
    fs::create_dir_all(&rolling_stock_a_tpl_path.parent().unwrap()).unwrap();
    fs::write(&rolling_stock_a_tpl_path, fs::read_to_string(ROLLING_STOCK_A_TPL_PATH).unwrap()).unwrap();

    let rolling_stock_b_tpl_path = tmp_dir.path().join("data_dir/dev/rolling-stock-b.trn");
    fs::create_dir_all(&rolling_stock_b_tpl_path.parent().unwrap()).unwrap();
    fs::write(&rolling_stock_b_tpl_path, fs::read_to_string(ROLLING_STOCK_B_TPL_PATH).unwrap()).unwrap();

    let route_1_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-1.trn");
    fs::create_dir_all(&route_1_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_1_tpl_path, fs::read_to_string(ROUTE_1_TPL_PATH).unwrap()).unwrap();

    let route_2_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-2.trn");
    fs::create_dir_all(&route_2_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_2_tpl_path, fs::read_to_string(ROUTE_2_TPL_PATH).unwrap()).unwrap();

    let route_1_2_schedule_path = tmp_dir.path().join("data_dir/dev/route-part-1-2.schedule.xml");
    fs::create_dir_all(&route_1_2_schedule_path.parent().unwrap()).unwrap();
    fs::write(&route_1_2_schedule_path, fs::read_to_string(ROUTE_1_2_SCHEDULE_PATH).unwrap()).unwrap();

    let fpn_path = tmp_dir.path().join("data_dir/out/my-fahrplan.fpn");
    let trn_1_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20000.trn");
    let trn_2_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20002.trn");
    let trn_3_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20004.trn");
    let trn_4_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20006.trn");

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

    assert_eq!(read_xml_file(&fpn_path), read_xml_file(EXPECTED_FPN_PATH));
    assert_eq!(read_xml_file(&trn_1_path), read_xml_file(EXPECTED_TRN_1_PATH));
    assert_eq!(read_xml_file(&trn_2_path), read_xml_file(EXPECTED_TRN_2_PATH));
    assert_eq!(read_xml_file(&trn_3_path), read_xml_file(EXPECTED_TRN_3_PATH));
    assert_eq!(read_xml_file(&trn_4_path), read_xml_file(EXPECTED_TRN_4_PATH));

    assert_eq!(fs::read_to_string(&config_path).unwrap(), fs::read_to_string(CONFIG_PATH).unwrap());
    assert_eq!(fs::read_to_string(&fpn_tpl_path).unwrap(), fs::read_to_string(FPN_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&meta_data_tpl_path).unwrap(), fs::read_to_string(META_DATA_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_a_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_A_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_b_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_B_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_tpl_path).unwrap(), fs::read_to_string(ROUTE_1_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_2_tpl_path).unwrap(), fs::read_to_string(ROUTE_2_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_2_schedule_path).unwrap(), fs::read_to_string(ROUTE_1_2_SCHEDULE_PATH).unwrap());

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
        config_path,
        meta_data_tpl_path,
        fpn_tpl_path,
        rolling_stock_a_tpl_path,
        rolling_stock_b_tpl_path,
        route_1_2_schedule_path,
        route_1_tpl_path,
        route_2_tpl_path,
        trn_1_path,
        trn_2_path,
        trn_3_path,
        trn_4_path,
        fpn_path,
    ]);
}