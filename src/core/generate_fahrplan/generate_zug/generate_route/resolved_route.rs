use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoute {
    pub aufgleis_fahrstrasse: String,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
}

impl From<ResolvedRoutePart> for ResolvedRoute {
    fn from(ResolvedRoutePart { aufgleis_fahrstrasse, fahrplan_eintraege, .. }: ResolvedRoutePart) -> Self {
        Self {
            aufgleis_fahrstrasse,
            fahrplan_eintraege,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoutePart {
    pub aufgleis_fahrstrasse: String,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
    pub has_time_fix: bool,
}

impl ResolvedRoutePart {
    pub fn new(aufgleis_fahrstrasse: String, fahrplan_eintraege: Vec<FahrplanEintrag>) -> Self {
        Self {
            aufgleis_fahrstrasse,
            fahrplan_eintraege,
            has_time_fix: false,
        }
    }
}