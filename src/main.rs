use clap::Parser;
use serde_helpers::xml::FromXML;
use zusi_fahrplan_generator::cli::{Cli, CliCommand};
use zusi_fahrplan_generator::core::generate_fahrplan::generate_fahrplan;
use zusi_fahrplan_generator::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
use zusi_fahrplan_generator::input::fahrplan_config::FahrplanConfig;

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        CliCommand::GenerateFahrplan(args) => {
            let config_path = args.config;
            println!("Generate Fahrplan using config file at {:?}", config_path);
            let config = ZusiEnvironmentConfig::<FahrplanConfig>::from_xml_file_by_path(&config_path)
                .map_err(|error| format!("Couldn't read the config file: {error}"))?;
            let (environment, fahrplan_config) = config.into_zusi_environment(config_path)
                .map_err(|error| format!("Couldn't create the ZusiEnvironment: {error}"))?;
            println!("{environment}");
            generate_fahrplan(&environment, fahrplan_config).map_err(|error| format!("{error}"))
        }
    }
}