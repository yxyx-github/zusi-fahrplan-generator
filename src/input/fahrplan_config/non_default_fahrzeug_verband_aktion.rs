use serde::{Deserialize, Serialize};
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::fahrzeug_verband_aktion::FahrzeugVerbandAktion;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum NonDefaultFahrzeugVerbandAktion {
    #[serde(rename = "1")]
    ZugDrehen,

    #[serde(rename = "2")]
    Fueherstandswechsel,
}

impl From<NonDefaultFahrzeugVerbandAktion> for FahrzeugVerbandAktion {
    fn from(value: NonDefaultFahrzeugVerbandAktion) -> Self {
        match value {
            NonDefaultFahrzeugVerbandAktion::ZugDrehen => FahrzeugVerbandAktion::ZugDrehen,
            NonDefaultFahrzeugVerbandAktion::Fueherstandswechsel => FahrzeugVerbandAktion::Fueherstandswechsel,
        }
    }
}