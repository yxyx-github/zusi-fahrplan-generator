mod utils;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use glob::glob;
use serde_helpers::xml::test_utils::read_xml_file;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;
use crate::utils::print_output;

const CONFIG_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/config.xml";
const FPN_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/my-fahrplan.xml";
const ROUTE_1_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/route-part-1.xml";
const ROUTE_1_TIMETABLE_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/route-part-1.timetable.xml";
const ROUTE_2_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/route-part-2.xml";
const ROUTE_2_TIMETABLE_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/route-part-2.timetable.xml";
const META_DATA_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/meta-data.xml";
const ROLLING_STOCK_A_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/rolling-stock-a.xml";
const ROLLING_STOCK_A_TIMETABLE_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/rolling-stock-a.timetable.xml";
const ROLLING_STOCK_B_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/rolling-stock-b.xml";
const ROLLING_STOCK_B_TIMETABLE_TPL_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/rolling-stock-b.timetable.xml";
const ROUTE_1_2_SCHEDULE_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/input/route-part-1-2.schedule.xml";

const EXPECTED_FPN_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/expected/my-fahrplan.xml";
const EXPECTED_TRN_1_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/expected/trn-1.xml";
const EXPECTED_TRN_1_TIMETABLE_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/expected/trn-1.timetable.xml";
const EXPECTED_TRN_2_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/expected/trn-2.xml";
const EXPECTED_TRN_2_TIMETABLE_PATH: &str = "./tests/generate_fahrplan_with_buchfahrplan/expected/trn-2.timetable.xml";

#[test]
fn test_generate_fahrplan_with_buchfahrplan() {
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

    let rolling_stock_a_timetable_tpl_path = tmp_dir.path().join("data_dir/dev/rolling-stock-a.timetable.xml");
    fs::create_dir_all(&rolling_stock_a_timetable_tpl_path.parent().unwrap()).unwrap();
    fs::write(&rolling_stock_a_timetable_tpl_path, fs::read_to_string(ROLLING_STOCK_A_TIMETABLE_TPL_PATH).unwrap()).unwrap();

    let rolling_stock_b_tpl_path = tmp_dir.path().join("data_dir/dev/rolling-stock-b.trn");
    fs::create_dir_all(&rolling_stock_b_tpl_path.parent().unwrap()).unwrap();
    fs::write(&rolling_stock_b_tpl_path, fs::read_to_string(ROLLING_STOCK_B_TPL_PATH).unwrap()).unwrap();

    let rolling_stock_b_timetable_tpl_path = tmp_dir.path().join("data_dir/dev/rolling-stock-b.timetable.xml");
    fs::create_dir_all(&rolling_stock_b_timetable_tpl_path.parent().unwrap()).unwrap();
    fs::write(&rolling_stock_b_timetable_tpl_path, fs::read_to_string(ROLLING_STOCK_B_TIMETABLE_TPL_PATH).unwrap()).unwrap();

    let route_1_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-1.trn");
    fs::create_dir_all(&route_1_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_1_tpl_path, fs::read_to_string(ROUTE_1_TPL_PATH).unwrap()).unwrap();

    let route_1_timetable_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-1.timetable.xml");
    fs::create_dir_all(&route_1_timetable_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_1_timetable_tpl_path, fs::read_to_string(ROUTE_1_TIMETABLE_TPL_PATH).unwrap()).unwrap();

    let route_2_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-2.trn");
    fs::create_dir_all(&route_2_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_2_tpl_path, fs::read_to_string(ROUTE_2_TPL_PATH).unwrap()).unwrap();

    let route_2_timetable_tpl_path = tmp_dir.path().join("data_dir/dev/route-part-2.timetable.xml");
    fs::create_dir_all(&route_2_timetable_tpl_path.parent().unwrap()).unwrap();
    fs::write(&route_2_timetable_tpl_path, fs::read_to_string(ROUTE_2_TIMETABLE_TPL_PATH).unwrap()).unwrap();

    let route_1_2_schedule_path = tmp_dir.path().join("data_dir/dev/route-part-1-2.schedule.xml");
    fs::create_dir_all(&route_1_2_schedule_path.parent().unwrap()).unwrap();
    fs::write(&route_1_2_schedule_path, fs::read_to_string(ROUTE_1_2_SCHEDULE_PATH).unwrap()).unwrap();

    let fpn_path = tmp_dir.path().join("data_dir/out/my-fahrplan.fpn");
    let trn_1_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20000.trn");
    let trn_1_timetable_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20000.timetable.xml");
    let trn_2_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20002.trn");
    let trn_2_timetable_path = tmp_dir.path().join("data_dir/out/my-fahrplan/RB20002.timetable.xml");

    let mut cmd = Command::cargo_bin("zusi-fahrplan-generator").unwrap();
    let output = cmd.arg("generate-fahrplan").arg("-c").arg(&config_path)
        .assert()
        /*.stdout(predicates::str::is_match(concat!(
            r#"^Generate Fahrplan using config file at "/tmp/\.[a-zA-Z0-9]+/data_dir/dev/config.xml"\r?\n"#,
            r#"Zusi data dir: "/tmp/.[a-zA-Z0-9]+/data_dir"\r?\n"#,
            r#"Config dir: "/tmp/.[a-zA-Z0-9]+/data_dir/dev"(\r\n|\n)*$"#,
        )).unwrap())*/
        .stderr("")
        .success().get_output().clone();

    print_output(output);

    assert_eq!(read_xml_file(&fpn_path), read_xml_file(EXPECTED_FPN_PATH));
    assert_eq!(read_xml_file(&trn_1_path), read_xml_file(EXPECTED_TRN_1_PATH)); // TODO either BremsstellungZug missing or MBrh shouldn't be added
    assert_eq!(read_xml_file(&trn_1_timetable_path), read_xml_file(EXPECTED_TRN_1_TIMETABLE_PATH));
    assert_eq!(read_xml_file(&trn_2_path), read_xml_file(EXPECTED_TRN_2_PATH)); // TODO either BremsstellungZug missing or MBrh shouldn't be added
    assert_eq!(read_xml_file(&trn_2_timetable_path), read_xml_file(EXPECTED_TRN_2_TIMETABLE_PATH));

    assert_eq!(fs::read_to_string(&config_path).unwrap(), fs::read_to_string(CONFIG_PATH).unwrap());
    assert_eq!(fs::read_to_string(&fpn_tpl_path).unwrap(), fs::read_to_string(FPN_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&meta_data_tpl_path).unwrap(), fs::read_to_string(META_DATA_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_a_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_A_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_a_timetable_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_A_TIMETABLE_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_b_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_B_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&rolling_stock_b_timetable_tpl_path).unwrap(), fs::read_to_string(ROLLING_STOCK_B_TIMETABLE_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_tpl_path).unwrap(), fs::read_to_string(ROUTE_1_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_1_timetable_tpl_path).unwrap(), fs::read_to_string(ROUTE_1_TIMETABLE_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_2_tpl_path).unwrap(), fs::read_to_string(ROUTE_2_TPL_PATH).unwrap());
    assert_eq!(fs::read_to_string(&route_2_timetable_tpl_path).unwrap(), fs::read_to_string(ROUTE_2_TIMETABLE_TPL_PATH).unwrap());
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
        rolling_stock_a_timetable_tpl_path,
        rolling_stock_a_tpl_path,
        rolling_stock_b_timetable_tpl_path,
        rolling_stock_b_tpl_path,
        route_1_2_schedule_path,
        route_1_timetable_tpl_path,
        route_1_tpl_path,
        route_2_timetable_tpl_path,
        route_2_tpl_path,
        trn_1_timetable_path,
        trn_1_path,
        trn_2_timetable_path,
        trn_2_path,
        fpn_path,
    ]);
}