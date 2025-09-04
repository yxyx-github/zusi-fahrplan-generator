use clap::Parser;
use serde_helpers::xml::FromXML;
use zusi_fahrplan_generator::cli::{Cli, Command};
use zusi_fahrplan_generator::core::fahrplan_generator::generate_fahrplan;
use zusi_fahrplan_generator::input::environment::zusi_environment_config::ZusiEnvironmentConfig;
use zusi_fahrplan_generator::input::fahrplan_config::FahrplanConfig;

fn main() {
    fn main() {
        let cli = Cli::parse();

        match cli.command {
            Command::GenerateFahrplan(args) => {
                println!("Generate Fahrplan using config file at {:?}", args.config);
                let config = ZusiEnvironmentConfig::<FahrplanConfig>::from_xml_file_by_path(&args.config).unwrap();
                let (environment, fahrplan_config) = config.into_zusi_environment(args.config);
                generate_fahrplan(&environment, fahrplan_config).unwrap_or_else(|error| eprintln!("{error}"));
            }
        };
    }
}
