use clap::Parser;
use serde_helpers::xml::{FromXML, ToXML};
use std::path::Path;
use zusi_fahrplan_generator::cli::{Cli, CliCommand, CliScheduleCommand};
use zusi_fahrplan_generator::core::generate_fahrplan::generate_fahrplan;
use zusi_fahrplan_generator::core::lib::helpers::read_zug;
use zusi_fahrplan_generator::core::schedules::apply::apply_schedule;
use zusi_fahrplan_generator::core::schedules::generate::generate_schedule;
use zusi_fahrplan_generator::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
use zusi_fahrplan_generator::input::fahrplan_config::FahrplanConfig;
use zusi_fahrplan_generator::input::schedule::Schedule;
use zusi_xml_lib::xml::zusi::Zusi;

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        CliCommand::GenerateFahrplan(args) => {
            let config_path = args.config;

            println!(r#"Generate Fahrplan using config file at "{}""#, config_path.display());
            let config = ZusiEnvironmentConfig::<FahrplanConfig>::from_xml_file_by_path(&config_path)
                .map_err(|error| format!("Couldn't read the config file: {error}"))?;
            let (environment, fahrplan_config) = config.into_zusi_environment(config_path)
                .map_err(|error| format!("Couldn't create the ZusiEnvironment: {error}"))?;
            println!("{environment}");
            generate_fahrplan(&environment, fahrplan_config).map_err(|error| format!("{error}"))
        },
        CliCommand::Schedule(CliScheduleCommand::Apply(args)) => {
            let schedule_path = args.schedule;
            let trn_file_paths = args.trn_files;

            let schedule = Schedule::from_xml_file_by_path(schedule_path)
                .map_err(|error| format!("Couldn't read the schedule file: {error}"))?;
            trn_file_paths
                .into_iter()
                .for_each(|trn_file_path| {
                    apply_schedule_to_file(&schedule, &trn_file_path) // TODO: update_buchfahrplan if BuchfahrplanDatei is present? (currently impossible due to missing data_dir)
                        .unwrap_or_else(|error| eprintln!(r#"Error occoured for "{}": {error}"#, trn_file_path.display()));
                });
            Ok(())
        },
        CliCommand::Schedule(CliScheduleCommand::Generate(args)) => {
            let schedule_path = args.schedule;
            let trn_file_path = args.trn;

            let zug = read_zug(trn_file_path)
                .map_err(|error| format!(r"Couldn't read the trn file: {error}"))?;
            let schedule = generate_schedule(&zug.value.fahrplan_eintraege);
            schedule.to_xml_file_by_path(schedule_path, true).map_err(|error| format!(r"Couldn't write the schedule file: {error}"))
        }
    }
}

fn apply_schedule_to_file<P: AsRef<Path>>(schedule: &Schedule, trn_file_path: P) -> Result<(), String> {
    let trn_file_path = trn_file_path.as_ref();
    let mut zug = read_zug(trn_file_path)
        .map_err(|error| format!(r"Couldn't read the trn file: {error}"))?;
    apply_schedule(&mut zug.value.fahrplan_eintraege, schedule)
        .map_err(|error| format!(r"Couldn't apply the schedule: {error}"))?;
    let zug: Zusi = zug.into();
    zug.to_xml_file_by_path(trn_file_path, true)
        .map_err(|error| format!(r"Couldn't write the trn file: {error}"))?;
    Ok(())
}