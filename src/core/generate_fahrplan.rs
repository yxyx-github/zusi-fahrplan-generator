mod generate_zug;

use crate::core::lib::generated_zug::GeneratedZug;
use crate::core::generate_fahrplan::generate_zug::{generate_zug, GenerateZugError};
use crate::core::generate_fahrplan::GenerateFahrplanError::ReadFahrplanTemplateError;
use crate::core::lib::file_error::FileError;
use crate::core::lib::helpers::{datei_from_prejoined_zusi_path, generate_buchfahrplan_path, generate_zug_path, read_fahrplan};
use crate::core::lib::zug_nummer::ZugNummer;
use crate::input::environment::zusi_environment::ZusiEnvironment;
use crate::input::fahrplan_config::FahrplanConfig;
use serde_helpers::xml::ToXML;
use thiserror::Error;
use zusi_xml_lib::xml::zusi::fahrplan::zug_datei_eintrag::ZugDateiEintrag;
use zusi_xml_lib::xml::zusi::fahrplan::Fahrplan;
use zusi_xml_lib::xml::zusi::lib::path::prejoined_zusi_path::PrejoinedZusiPath;
use zusi_xml_lib::xml::zusi::{TypedZusi, Zusi};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GenerateFahrplanError {
    #[error("The given Fahrplan template couldn't be read: {error}")]
    ReadFahrplanTemplateError {
        error: FileError,
    },

    #[error("The generated Fahrplan couldn't be written to disk: {error}")]
    WriteGeneratedFahrplanError {
        error: FileError,
    },

    #[error("A Zug couldn't be generated: {error}")]
    GenerateZugError {
        error: GenerateZugError,
    },

    #[error("A Zug couldn't be attached: {error}")]
    AttachZugError {
        error: FileError,
    },
}

impl From<GenerateZugError> for GenerateFahrplanError {
    fn from(error: GenerateZugError) -> Self {
        GenerateFahrplanError::GenerateZugError { error }
    }
}

pub fn generate_fahrplan(env: &ZusiEnvironment, config: FahrplanConfig) -> Result<(), GenerateFahrplanError> {
    let generate_from = env.path_to_prejoined_zusi_path(&config.generate_from)
        .map_err(|error| GenerateFahrplanError::ReadFahrplanTemplateError { error })?;
    let generate_at = env.path_to_prejoined_zusi_path(&config.generate_at)
        .map_err(|error| GenerateFahrplanError::WriteGeneratedFahrplanError { error })?;

    let mut fahrplan = read_fahrplan(generate_from.full_path())
        .map_err(|error| ReadFahrplanTemplateError { error })?;

    // any existing trains should be discarded
    fahrplan.value.trn_dateien = true;
    fahrplan.value.zug_dateien = vec![];
    fahrplan.value.zug_eintraege = vec![];

    let zuege = config.zuege
        .into_iter()
        .map(|train| generate_zug(env, &generate_at, train))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let zuege = sort_zuege(zuege);
    zuege
        .into_iter()
        .try_for_each(|zug| attach_zug(&mut fahrplan, zug, &generate_at))?;

    let fahrplan: Zusi = fahrplan.into();
    fahrplan.to_xml_file_by_path(generate_at.full_path(), true)
        .map_err(|error| GenerateFahrplanError::WriteGeneratedFahrplanError { error: (generate_at.full_path(), error).into() })?;

    Ok(())
}

fn attach_zug(fahrplan: &mut TypedZusi<Fahrplan>, mut zug: GeneratedZug, fahrplan_path: &PrejoinedZusiPath) -> Result<(), GenerateFahrplanError> {
    let zug_path = generate_zug_path(&zug.zug, fahrplan_path);

    if let Some(mut buchfahrplan) = zug.buchfahrplan {
        let buchfahrplan_path = generate_buchfahrplan_path(&buchfahrplan, fahrplan_path);

        buchfahrplan.value.datei_fpn = datei_from_prejoined_zusi_path(fahrplan_path, true)
            .map_err(|error| GenerateFahrplanError::AttachZugError { error })?;
        buchfahrplan.value.datei_trn = datei_from_prejoined_zusi_path(&zug_path, true)
            .map_err(|error| GenerateFahrplanError::AttachZugError { error })?;
        buchfahrplan.value.utm = fahrplan.value.utm.clone();

        zug.zug.value.buchfahrplan_roh_datei = Some(datei_from_prejoined_zusi_path(&buchfahrplan_path, false)
            .map_err(|error| GenerateFahrplanError::AttachZugError { error })?);

        let buchfahrplan: Zusi = buchfahrplan.into();
        buchfahrplan.to_xml_file_by_path(buchfahrplan_path.full_path(), true)
            .map_err(|error| GenerateFahrplanError::AttachZugError { error: (buchfahrplan_path.full_path(), error).into() })?;
    }

    let zug: Zusi = zug.zug.into();
    fahrplan.value.zug_dateien.push(
        ZugDateiEintrag::builder()
            .datei(
                datei_from_prejoined_zusi_path(&zug_path, false)
                    .map_err(|error| GenerateFahrplanError::AttachZugError { error })?
            )
            .build()
    );
    zug.to_xml_file_by_path(zug_path.full_path(), true)
        .map_err(|error| GenerateFahrplanError::AttachZugError { error: (zug_path.full_path(), error).into() })?;
    Ok(())
}

fn sort_zuege(zuege: Vec<GeneratedZug>) -> Vec<GeneratedZug> {
    let mut zuege: Vec<(ZugNummer, GeneratedZug)> = zuege
        .into_iter()
        .map(|zug| (zug.zug.value.nummer.clone().try_into().unwrap_or_default(), zug))
        .collect();
    zuege.sort_by(|zug1, zug2| zug1.0.cmp(&zug2.0));
    zuege.into_iter().map(|zug| zug.1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::fahrplan_config::{RouteConfig, RoutePart, RoutePartSource, ZugConfig};
    use crate::input::rolling_stock_config::RollingStockConfig;
    use glob::glob;
    use serde_helpers::xml::test_utils::{cleanup_xml, read_xml_file};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    const FROM_FPN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Fahrplan" Version="A.5" MinVersion="A.1"/>
            <Fahrplan AnfangsZeit="2024-06-20 07:30:00" ChaosVorschlagen="1" trnDateien="1">
                <BefehlsKonfiguration Dateiname="Signals\Deutschland\Befehle\408_2015.authority.xml"/>
                <LaPDF/>
                <StrebuPDF/>
                <ErsatzfahrplaenePDF/>
                <Begruessungsdatei/>
                <Zug>
                    <Datei Dateiname="any/train/here/is/ignored/RE3.trn"/>
                </Zug>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0005_0058\000541_005773_Voldagsen\Voldagsen_2004.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0005_0058\000546_005773_Osterwald\Osterwald_2004.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0006_0058\000551_005775_Elze\Elze_2003.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
            </Fahrplan>
        </Zusi>
    "#;

    const EXPECTED_FPN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Fahrplan" Version="A.5" MinVersion="A.1"/>
            <Fahrplan AnfangsZeit="2024-06-20 07:30:00" ChaosVorschlagen="1" trnDateien="1">
                <BefehlsKonfiguration Dateiname="Signals\Deutschland\Befehle\408_2015.authority.xml"/>
                <LaPDF/>
                <StrebuPDF/>
                <ErsatzfahrplaenePDF/>
                <Begruessungsdatei/>
                <Zug>
                    <Datei Dateiname="test/out/test/RB10001.trn"/>
                </Zug>
                <Zug>
                    <Datei Dateiname="test/out/test/RB20001.trn"/>
                </Zug>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0005_0058\000541_005773_Voldagsen\Voldagsen_2004.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0005_0058\000546_005773_Osterwald\Osterwald_2004.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <StrModul>
                    <Datei Dateiname="Routes\Deutschland\32U_0006_0058\000551_005775_Elze\Elze_2003.st3"/>
                    <p/>
                    <phi/>
                </StrModul>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
            </Fahrplan>
        </Zusi>
    "#;

    const ROUTE1_TEMPLATE_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROUTE1_TEMPLATE_TRN_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug MBrh="1.7" FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="test/dev/test/RB10001.timetable.xml"/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROUTE1_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0"/>
            <Buchfahrplan Gattung="RB" Nummer="00002">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
                <FplZeile FplLaufweg="20092.018">
                    <Fplkm km="32.8757" />
                    <FplName FplNameText="Elze" />
                    <FplAnk Ank="2024-06-20 08:39:00" />
                    <FplAbf Abf="2024-06-20 08:41:40" />
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="21799.445">
                    <FplvMax vMax="33.3333"/>
                    <Fplkm km="1.7792"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="24631.027">
                    <Fplkm km="4.5357"/>
                    <FplName FplNameText="Mehle Hp"/>
                    <FplAbf Abf="2024-06-20 08:45:00"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                    <Fplkm km="9.0405"/>
                    <FplName FplNameText="Osterwald Hp"/>
                    <FplAnk Ank="2024-06-20 08:48:00"/>
                    <FplAbf Abf="2024-06-20 08:48:40"/>
                </FplZeile>
            </Buchfahrplan>
        </Zusi>
    "#;

    const ROUTE2_TEMPLATE_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrplanEintrag Betrst="Voldagsen" FplEintrag="1">
                    <FahrplanSignalEintrag FahrplanSignal="A"/>
                </FahrplanEintrag>
                <FahrplanEintrag Ank="2024-06-20 08:52:10" Abf="2024-06-20 08:52:50" Signalvorlauf="160" Betrst="Voldagsen">
                    <FahrplanSignalEintrag FahrplanSignal="N2"/>
                </FahrplanEintrag>
                <FahrzeugVarianten/>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TEMPLATE_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug>
                <Datei/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TEMPLATE_TRN_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.5" MinVersion="A.1"/>
            <Zug>
                <Datei/>
                <BuchfahrplanRohDatei Dateiname="test/dev/test/rolling-stock/Triebwagen-A.timetable.xml"/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const ROLLING_STOCK_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="utf-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.7" MinVersion="A.0" />
            <Buchfahrplan Gattung="RE" Nummer="99999" spMax="20" MBrh="1.4" BremsstellungZug="3">
                <Datei_fpn/>
                <Datei_trn/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
            </Buchfahrplan>
        </Zusi>
    "#;

    const EXPECTED_ROUTE1_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.6" MinVersion="A.6"/>
            <Zug Gattung="RB" Nummer="10001" FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei Dateiname="test/out/test.fpn" NurInfo="1"/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const EXPECTED_ROUTE1_TRN_WITH_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.6" MinVersion="A.6"/>
            <Zug Gattung="RB" Nummer="10001" BremsstellungZug="3" MBrh="1.7" FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei Dateiname="test/out/test.fpn" NurInfo="1"/>
                <BuchfahrplanRohDatei Dateiname="test/out/test/RB10001.timetable.xml"/>
                <FahrplanEintrag Ank="2024-06-20 08:39:00" Abf="2024-06-20 08:41:40" Signalvorlauf="180" Betrst="Elze">
                    <FahrplanSignalEintrag FahrplanSignal="N1"/>
                </FahrplanEintrag>
                <FahrplanEintrag Abf="2024-06-20 08:45:00" Betrst="Mehle Hp"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    const EXPECTED_ROUTE1_TIMETABLE: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Buchfahrplan" Version="A.4" MinVersion="A.4"/>
            <Buchfahrplan Gattung="RB" Nummer="10001" spMax="20" MBrh="1.7" BremsstellungZug="3">
                <Datei_fpn Dateiname="test/out/test.fpn" NurInfo="1"/>
                <Datei_trn Dateiname="test/out/test/RB10001.trn" NurInfo="1"/>
                <UTM UTM_WE="566" UTM_NS="5793" UTM_Zone="32" UTM_Zone2="U"/>
                <FplZeile FplLaufweg="20092.018">
                    <Fplkm km="32.8757" />
                    <FplName FplNameText="Elze" />
                    <FplAnk Ank="2024-06-20 08:39:00" />
                    <FplAbf Abf="2024-06-20 08:41:40" />
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="21799.445">
                    <FplvMax vMax="33.3333"/>
                    <Fplkm km="1.7792"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="24631.027">
                    <Fplkm km="4.5357"/>
                    <FplName FplNameText="Mehle Hp"/>
                    <FplAbf Abf="2024-06-20 08:45:00"/>
                </FplZeile>
                <FplZeile FplRglGgl="1" FplLaufweg="29134.139">
                    <Fplkm km="9.0405"/>
                    <FplName FplNameText="Osterwald Hp"/>
                    <FplAnk Ank="2024-06-20 08:48:00"/>
                    <FplAbf Abf="2024-06-20 08:48:40"/>
                </FplZeile>
            </Buchfahrplan>
        </Zusi>
    "#;

    const EXPECTED_ROUTE2_TRN: &str = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <Zusi>
            <Info DateiTyp="Zug" Version="A.6" MinVersion="A.6"/>
            <Zug Gattung="RB" Nummer="20001" FahrstrName="Aufgleispunkt -&gt; Hildesheim Hbf F">
                <Datei Dateiname="test/out/test.fpn" NurInfo="1"/>
                <FahrplanEintrag Ank="2024-06-20 08:48:00" Abf="2024-06-20 08:48:40" Signalvorlauf="160" Betrst="Osterwald Hp"/>
                <FahrplanEintrag Betrst="Voldagsen" FplEintrag="1">
                    <FahrplanSignalEintrag FahrplanSignal="A"/>
                </FahrplanEintrag>
                <FahrplanEintrag Ank="2024-06-20 08:52:10" Abf="2024-06-20 08:52:50" Signalvorlauf="160" Betrst="Voldagsen">
                    <FahrplanSignalEintrag FahrplanSignal="N2"/>
                </FahrplanEintrag>
                <FahrzeugVarianten Bezeichnung="default" ZufallsWert="1">
                    <FahrzeugInfo IDHaupt="1" IDNeben="1">
                        <Datei Dateiname="TriebwagenA.fzg"/>
                    </FahrzeugInfo>
                </FahrzeugVarianten>
            </Zug>
        </Zusi>
    "#;

    #[test]
    fn test_generate_fahrplan() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let from_fpn_path = tmp_dir.path().join("test/dev/test.fpn");
        fs::create_dir_all(from_fpn_path.parent().unwrap()).unwrap();
        fs::write(&from_fpn_path, FROM_FPN).unwrap();

        let at_fpn_path = tmp_dir.path().join("test/out/test.fpn");

        let route1_path = tmp_dir.path().join("test/out/test/RB10001.trn");
        let route1_template_path = tmp_dir.path().join("test/dev/test/RB10001.trn");
        fs::create_dir_all(route1_template_path.parent().unwrap()).unwrap();
        fs::write(&route1_template_path, ROUTE1_TEMPLATE_TRN).unwrap();

        let route2_path = tmp_dir.path().join("test/out/test/RB20001.trn");
        let route2_template_path = tmp_dir.path().join("test/dev/test/RB20001.trn");
        fs::create_dir_all(route2_template_path.parent().unwrap()).unwrap();
        fs::write(&route2_template_path, ROUTE2_TEMPLATE_TRN).unwrap();

        let rolling_stock_path = tmp_dir.path().join("test/dev/test/rolling-stock/Triebwagen-A.trn");
        fs::create_dir_all(rolling_stock_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_path, ROLLING_STOCK_TEMPLATE_TRN).unwrap();

        let config = FahrplanConfig {
            generate_at: at_fpn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            generate_from: from_fpn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            zuege: vec![
                ZugConfig {
                    nummer: "20001".into(),
                    gattung: "RB".into(),
                    meta_data: None,
                    route: RouteConfig {
                        parts: vec![
                            RoutePart {
                                source: RoutePartSource::TrainFileByPath { path: route2_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                                time_fix: None,
                                apply_schedule: None,
                            },
                        ],
                    },
                    rolling_stock: RollingStockConfig {
                        path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
                    },
                    copy_delay_config: None,
                },
                ZugConfig {
                    nummer: "10001".into(),
                    gattung: "RB".into(),
                    meta_data: None,
                    route: RouteConfig {
                        parts: vec![
                            RoutePart {
                                source: RoutePartSource::TrainFileByPath { path: route1_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                                time_fix: None,
                                apply_schedule: None,
                            },
                        ],
                    },
                    rolling_stock: RollingStockConfig {
                        path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
                    },
                    copy_delay_config: None,
                },
            ],
        };

        generate_fahrplan(&env, config).unwrap();

        assert_eq!(read_xml_file(&at_fpn_path), cleanup_xml(EXPECTED_FPN.into()));
        assert_eq!(read_xml_file(&route1_path), cleanup_xml(EXPECTED_ROUTE1_TRN.into()));
        assert_eq!(read_xml_file(&route2_path), cleanup_xml(EXPECTED_ROUTE2_TRN.into()));

        assert_eq!(fs::read_to_string(&from_fpn_path).unwrap(), FROM_FPN);
        assert_eq!(fs::read_to_string(&route1_template_path).unwrap(), ROUTE1_TEMPLATE_TRN);
        assert_eq!(fs::read_to_string(&route2_template_path).unwrap(), ROUTE2_TEMPLATE_TRN);
        assert_eq!(fs::read_to_string(&rolling_stock_path).unwrap(), ROLLING_STOCK_TEMPLATE_TRN);

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
            route1_template_path,
            route2_template_path,
            rolling_stock_path,
            from_fpn_path,
            route1_path,
            route2_path,
            at_fpn_path,
        ]);
    }

    #[test]
    fn test_generate_fahrplan_with_buchfahrplan() {
        let tmp_dir = tempdir().unwrap();

        let env = ZusiEnvironment {
            data_dir: tmp_dir.path().to_owned(),
            config_dir: tmp_dir.path().to_owned(),
        };

        let from_fpn_path = tmp_dir.path().join("test/dev/test.fpn");
        fs::create_dir_all(from_fpn_path.parent().unwrap()).unwrap();
        fs::write(&from_fpn_path, FROM_FPN).unwrap();

        let at_fpn_path = tmp_dir.path().join("test/out/test.fpn");

        let route1_path = tmp_dir.path().join("test/out/test/RB10001.trn");
        let route1_template_path = tmp_dir.path().join("test/dev/test/RB10001.trn");
        fs::create_dir_all(route1_template_path.parent().unwrap()).unwrap();
        fs::write(&route1_template_path, ROUTE1_TEMPLATE_TRN_WITH_TIMETABLE).unwrap();

        let route1_timetable_path = tmp_dir.path().join("test/out/test/RB10001.timetable.xml");
        let route1_timetable_template_path = tmp_dir.path().join("test/dev/test/RB10001.timetable.xml");
        fs::create_dir_all(route1_timetable_template_path.parent().unwrap()).unwrap();
        fs::write(&route1_timetable_template_path, ROUTE1_TIMETABLE).unwrap();

        let route2_path = tmp_dir.path().join("test/out/test/RB20001.trn");
        let route2_template_path = tmp_dir.path().join("test/dev/test/RB20001.trn");
        fs::create_dir_all(route2_template_path.parent().unwrap()).unwrap();
        fs::write(&route2_template_path, ROUTE2_TEMPLATE_TRN).unwrap();

        let rolling_stock_path = tmp_dir.path().join("test/dev/test/rolling-stock/Triebwagen-A.trn");
        fs::create_dir_all(rolling_stock_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_path, ROLLING_STOCK_TEMPLATE_TRN_WITH_TIMETABLE).unwrap();

        let rolling_stock_timetable_path = tmp_dir.path().join("test/dev/test/rolling-stock/Triebwagen-A.timetable.xml");
        fs::create_dir_all(rolling_stock_timetable_path.parent().unwrap()).unwrap();
        fs::write(&rolling_stock_timetable_path, ROLLING_STOCK_TIMETABLE).unwrap();

        let config = FahrplanConfig {
            generate_at: at_fpn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            generate_from: from_fpn_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
            zuege: vec![
                ZugConfig {
                    nummer: "20001".into(),
                    gattung: "RB".into(),
                    meta_data: None,
                    route: RouteConfig {
                        parts: vec![
                            RoutePart {
                                source: RoutePartSource::TrainFileByPath { path: route2_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                                time_fix: None,
                                apply_schedule: None,
                            },
                        ],
                    },
                    rolling_stock: RollingStockConfig {
                        path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
                    },
                    copy_delay_config: None,
                },
                ZugConfig {
                    nummer: "10001".into(),
                    gattung: "RB".into(),
                    meta_data: None,
                    route: RouteConfig {
                        parts: vec![
                            RoutePart {
                                source: RoutePartSource::TrainFileByPath { path: route1_template_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned() },
                                time_fix: None,
                                apply_schedule: None,
                            },
                        ],
                    },
                    rolling_stock: RollingStockConfig {
                        path: rolling_stock_path.clone().strip_prefix(tmp_dir.path()).unwrap().to_owned(),
                    },
                    copy_delay_config: None,
                },
            ],
        };

        generate_fahrplan(&env, config).unwrap();

        assert_eq!(read_xml_file(&at_fpn_path), cleanup_xml(EXPECTED_FPN.into()));
        assert_eq!(read_xml_file(&route1_path), cleanup_xml(EXPECTED_ROUTE1_TRN_WITH_TIMETABLE.into()));
        assert_eq!(read_xml_file(&route1_timetable_path), cleanup_xml(EXPECTED_ROUTE1_TIMETABLE.into()));
        assert_eq!(read_xml_file(&route2_path), cleanup_xml(EXPECTED_ROUTE2_TRN.into()));

        assert_eq!(fs::read_to_string(&from_fpn_path).unwrap(), FROM_FPN);
        assert_eq!(fs::read_to_string(&route1_template_path).unwrap(), ROUTE1_TEMPLATE_TRN_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(&route1_timetable_template_path).unwrap(), ROUTE1_TIMETABLE);
        assert_eq!(fs::read_to_string(&route2_template_path).unwrap(), ROUTE2_TEMPLATE_TRN);
        assert_eq!(fs::read_to_string(&rolling_stock_path).unwrap(), ROLLING_STOCK_TEMPLATE_TRN_WITH_TIMETABLE);
        assert_eq!(fs::read_to_string(&rolling_stock_timetable_path).unwrap(), ROLLING_STOCK_TIMETABLE);

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
            route1_timetable_template_path,
            route1_template_path,
            route2_template_path,
            rolling_stock_timetable_path,
            rolling_stock_path,
            from_fpn_path,
            route1_timetable_path,
            route1_path,
            route2_path,
            at_fpn_path,
        ]);
    }
}