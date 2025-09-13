use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::{override_default, read_zug};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::MetaDataConfig;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::zug::Zug;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AddMetaDataError {
    #[error("Couldn't read the meta data template file: {error}")]
    ReadMetaDataError {
        #[from]
        error: FileError,
    },
}

pub fn add_meta_data(env: &ZusiEnvironment, config: MetaDataConfig, zug: &mut Zug) -> Result<(), AddMetaDataError> {
    let path = env.path_to_prejoined_zusi_path(config.path)?;
    let meta_data_template = read_zug(path.full_path())?;
    override_default(&mut zug.zuglauf, meta_data_template.value.zuglauf);
    override_default(&mut zug.prioritaet, meta_data_template.value.prioritaet);
    override_default(&mut zug.energie_vorgabe, meta_data_template.value.energie_vorgabe);
    override_default(&mut zug.mindest_bremshundertstel, meta_data_template.value.mindest_bremshundertstel);
    override_default(&mut zug.verkehrstage, meta_data_template.value.verkehrstage);
    override_default(&mut zug.speed_zug_niedriger, meta_data_template.value.speed_zug_niedriger);
    override_default(&mut zug.autopilot_beschleunigung, meta_data_template.value.autopilot_beschleunigung);
    override_default(&mut zug.keine_vorplan_korrektur, meta_data_template.value.keine_vorplan_korrektur);
    override_default(&mut zug.dekozug, meta_data_template.value.dekozug);
    override_default(&mut zug.lod_zug, meta_data_template.value.lod_zug);
    override_default(&mut zug.reisenden_dichte, meta_data_template.value.reisenden_dichte);
    override_default(&mut zug.fahrplan_gruppe, meta_data_template.value.fahrplan_gruppe);
    override_default(&mut zug.rekursionstiefe, meta_data_template.value.rekursionstiefe);
    override_default(&mut zug.zugsicherung_startmodus, meta_data_template.value.zugsicherung_startmodus);
    override_default(&mut zug.cold_movement, meta_data_template.value.cold_movement);
    override_default(&mut zug.zug_typ, meta_data_template.value.zug_typ);
    override_default(&mut zug.ueberschrift, meta_data_template.value.ueberschrift);
    override_default(&mut zug.buchfahrplan_einfach, meta_data_template.value.buchfahrplan_einfach);
    override_default(&mut zug.buchfahrplan_dll, meta_data_template.value.buchfahrplan_dll);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::generate_fahrplan::generate_zug::add_meta_data::add_meta_data;
    use crate::input::environment::zusi_environment::ZusiEnvironment;
    use crate::input::fahrplan_config::MetaDataConfig;
    use std::fs;
    use tempfile::tempdir;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::zug::fahrzeug_varianten::FahrzeugVarianten;
    use zusi_xml_lib::xml::zusi::zug::Zug;

    const META_DATA_TEMPLATE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug Zuglauf="ADorf - BDorf" FahrplanGruppe="Gruppe AB">
                <Datei/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    #[test]
    fn test_add_meta_data() {
        let tmp_dir = tempdir().unwrap();

        let meta_data_template_path = tmp_dir.path().join("meta-data.trn");
        fs::write(&meta_data_template_path, META_DATA_TEMPLATE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let mut zug = Zug::builder()
            .fahrplan_datei(Datei::builder().build())
            .fahrplan_gruppe("Gruppe A - B".into())
            .fahrzeug_varianten(FahrzeugVarianten::builder().build())
            .build();

        let config = MetaDataConfig {
            path: meta_data_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = Zug::builder()
            .fahrplan_datei(Datei::builder().build())
            .zuglauf("ADorf - BDorf".into())
            .fahrplan_gruppe("Gruppe A - B".into())
            .fahrzeug_varianten(FahrzeugVarianten::builder().build())
            .build();

        add_meta_data(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(meta_data_template_path).unwrap(), META_DATA_TEMPLATE);
    }
}