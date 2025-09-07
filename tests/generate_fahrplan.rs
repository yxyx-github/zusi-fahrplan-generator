use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::output::OutputOkExt;
use serde_helpers::xml::test_utils::read_xml_file;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

const CONFIG_PATH: &str = "./tests/generate_fahrplan/input/config.xml";
const FPN_TPL_PATH: &str = "./tests/generate_fahrplan/input/my-fahrplan.xml";
const EXPECTED_FPN_PATH: &str = "./tests/generate_fahrplan/expected/my-fahrplan.xml";
const ROUTE_1_TPL_PATH: &str = "./tests/generate_fahrplan/input/route-part-1.xml";
const ROUTE_2_TPL_PATH: &str = "./tests/generate_fahrplan/input/route-part-2.xml";
const ROLLING_STOCK_A_TPL_PATH: &str = "./tests/generate_fahrplan/input/rolling-stock-a.xml";
const ROLLING_STOCK_B_TPL_PATH: &str = "./tests/generate_fahrplan/input/rolling-stock-b.xml";
const ROUTE_1_SCHEDULE_PATH: &str = "./tests/generate_fahrplan/input/route-part-1.schedule.xml";

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

    let at_fpn_path = tmp_dir.path().join("data_dir/out/my-fahrplan.fpn");

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

    let route_1_schedule_path = tmp_dir.path().join("data_dir/dev/route-part-1.schedule.xml");
    fs::create_dir_all(&route_1_schedule_path.parent().unwrap()).unwrap();
    fs::write(&route_1_schedule_path, fs::read_to_string(ROUTE_1_SCHEDULE_PATH).unwrap()).unwrap();

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    cmd.arg("generate-fahrplan").arg("-c").arg(&config_path).assert().success();
    // output = cmd.arg("generate-fahrplan").arg("-c").arg(&config_path).output().unwrap().clone();
    // let output = cmd.arg("generate-fahrplan").arg("-c").arg(&config_path).assert().success().get_output().clone();

    /*println!("Output (stdout, stderr):");
    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!("{}", String::from_utf8(output.stderr).unwrap());*/

    assert_eq!(read_xml_file(&at_fpn_path), read_xml_file(EXPECTED_FPN_PATH));

    assert_eq!(fs::read_to_string(&config_path).unwrap(), fs::read_to_string(CONFIG_PATH).unwrap());
    assert_eq!(fs::read_to_string(&fpn_tpl_path).unwrap(), fs::read_to_string(FPN_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_a_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_A_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_b_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_B_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_tpl_path).unwrap(), fs::read_to_string(ROUTE_1_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_2_tpl_path).unwrap(), fs::read_to_string(ROUTE_2_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_schedule_path).unwrap(), fs::read_to_string(ROUTE_1_SCHEDULE_PATH).unwrap());
}