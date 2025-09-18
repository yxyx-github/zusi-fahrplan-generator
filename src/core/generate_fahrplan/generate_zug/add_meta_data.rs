use crate::core::lib::file_error::FileError;
use crate::core::lib::generated_zug::RawGeneratedZug;
use crate::core::lib::helpers::{override_default, read_zug};
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::MetaDataConfig;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AddMetaDataError {
    #[error("Couldn't read the meta data template file: {error}")]
    ReadMetaDataError {
        #[from]
        error: FileError,
    },
}

// TODO: add arg to specify if existing meta data should be overridden (currently not the case)
pub fn add_meta_data(env: &ZusiEnvironment, config: MetaDataConfig, zug: &mut RawGeneratedZug) -> Result<(), AddMetaDataError> {
    let path = env.path_to_prejoined_zusi_path(config.path)?;
    let meta_data_template = read_zug(path.full_path())?.value;

    override_default(&mut zug.zug.zuglauf, meta_data_template.zuglauf);
    override_default(&mut zug.zug.verkehrstage, meta_data_template.verkehrstage);
    override_default(&mut zug.zug.prioritaet, meta_data_template.prioritaet);
    override_default(&mut zug.zug.energie_vorgabe, meta_data_template.energie_vorgabe);
    override_default(&mut zug.zug.autopilot_beschleunigung, meta_data_template.autopilot_beschleunigung);
    override_default(&mut zug.zug.keine_vorplan_korrektur, meta_data_template.keine_vorplan_korrektur);
    override_default(&mut zug.zug.dekozug, meta_data_template.dekozug);
    override_default(&mut zug.zug.lod_zug, meta_data_template.lod_zug);
    override_default(&mut zug.zug.reisenden_dichte, meta_data_template.reisenden_dichte);
    override_default(&mut zug.zug.fahrplan_gruppe, meta_data_template.fahrplan_gruppe);
    override_default(&mut zug.zug.rekursionstiefe, meta_data_template.rekursionstiefe);
    override_default(&mut zug.zug.zugsicherung_startmodus, meta_data_template.zugsicherung_startmodus);
    override_default(&mut zug.zug.cold_movement, meta_data_template.cold_movement);
    override_default(&mut zug.zug.zug_typ, meta_data_template.zug_typ);
    override_default(&mut zug.zug.ueberschrift, meta_data_template.ueberschrift);
    override_default(&mut zug.zug.odt_datei_absolut, meta_data_template.odt_datei_absolut);
    override_default(&mut zug.zug.buchfahrplan_einfach, meta_data_template.buchfahrplan_einfach);
    override_default(&mut zug.zug.buchfahrplan_dll, meta_data_template.buchfahrplan_dll);
    override_default(&mut zug.zug.tuer_system_bezeichner, meta_data_template.tuer_system_bezeichner);

    if let Some(buchfahrplan) = &mut zug.buchfahrplan {
        buchfahrplan.gattung = zug.zug.gattung.clone();
        buchfahrplan.nummer = zug.zug.nummer.clone();
        buchfahrplan.zuglauf = zug.zug.zuglauf.clone();
        buchfahrplan.verkehrstage = zug.zug.verkehrstage.clone();

        override_default(&mut buchfahrplan.mindest_bremshundertstel, zug.zug.mindest_bremshundertstel);
        override_default(&mut buchfahrplan.laenge, zug.zug.fahrplan_zug_laenge);
        override_default(&mut buchfahrplan.baureihe, zug.zug.baureihe_angabe.clone());
        override_default(&mut buchfahrplan.bremsstellung_zug, zug.zug.bremsstellung_zug.clone());
        override_default(&mut buchfahrplan.fahrplan_bremsstellung_textvorgabe, zug.zug.fahrplan_bremsstellung_textvorgabe.clone());
        override_default(&mut buchfahrplan.masse, zug.zug.fahrplan_masse);
        override_default(&mut buchfahrplan.grenzlast, zug.zug.grenzlast);
        override_default(&mut buchfahrplan.speed_max, zug.zug.speed_zug_niedriger);

        // TODO: consider Buchfahrplan of meta_data_template if exists
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::generate_fahrplan::generate_zug::add_meta_data::add_meta_data;
    use crate::core::lib::generated_zug::RawGeneratedZug;
    use crate::input::environment::zusi_environment::ZusiEnvironment;
    use crate::input::fahrplan_config::MetaDataConfig;
    use std::fs;
    use tempfile::tempdir;
    use zusi_xml_lib::xml::zusi::buchfahrplan::Buchfahrplan;
    use zusi_xml_lib::xml::zusi::lib::datei::Datei;
    use zusi_xml_lib::xml::zusi::lib::utm::UTM;
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

        let mut zug = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .fahrplan_gruppe("Gruppe A - B".into())
                .fahrzeug_varianten(FahrzeugVarianten::builder().build())
                .build(),
            buchfahrplan: None,
        };

        let config = MetaDataConfig {
            path: meta_data_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .zuglauf("ADorf - BDorf".into())
                .fahrplan_gruppe("Gruppe A - B".into())
                .fahrzeug_varianten(FahrzeugVarianten::builder().build())
                .build(),
            buchfahrplan: None,
        };

        add_meta_data(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(meta_data_template_path).unwrap(), META_DATA_TEMPLATE);
    }

    #[test]
    fn test_add_meta_data_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let meta_data_template_path = tmp_dir.path().join("meta-data.trn");
        fs::write(&meta_data_template_path, META_DATA_TEMPLATE).unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let mut zug = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .fahrplan_gruppe("Gruppe A - B".into())
                .fahrzeug_varianten(FahrzeugVarianten::builder().build())
                .build(),
            buchfahrplan: Some(Buchfahrplan::builder()
                .datei_trn(Datei::builder().build())
                .datei_fpn(Datei::builder().build())
                .utm(UTM::builder().build())
                .zuglauf("A - B".into())
                .build()),
        };

        let config = MetaDataConfig {
            path: meta_data_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
        };

        let expected = RawGeneratedZug {
            zug: Zug::builder()
                .fahrplan_datei(Datei::builder().build())
                .zuglauf("ADorf - BDorf".into())
                .fahrplan_gruppe("Gruppe A - B".into())
                .fahrzeug_varianten(FahrzeugVarianten::builder().build())
                .build(),
            buchfahrplan: Some(Buchfahrplan::builder()
                .datei_trn(Datei::builder().build())
                .datei_fpn(Datei::builder().build())
                .utm(UTM::builder().build())
                .zuglauf("ADorf - BDorf".into())
                .build()),
        };

        add_meta_data(&env, config, &mut zug).unwrap();

        assert_eq!(zug, expected);

        assert_eq!(fs::read_to_string(meta_data_template_path).unwrap(), META_DATA_TEMPLATE);
    }
}