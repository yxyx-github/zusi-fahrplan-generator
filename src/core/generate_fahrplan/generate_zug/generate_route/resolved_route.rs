use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
use zusi_xml_lib::xml::zusi::zug::standort_modus::StandortModus;
use zusi_xml_lib::xml::zusi::zug::Zug;

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoute {
    pub start_data: RouteStartData,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
}

impl From<ResolvedRoutePart> for ResolvedRoute {
    fn from(ResolvedRoutePart { start_data, fahrplan_eintraege, .. }: ResolvedRoutePart) -> Self {
        Self {
            start_data,
            fahrplan_eintraege,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoutePart {
    pub start_data: RouteStartData,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
    pub has_time_fix: bool,
}

impl ResolvedRoutePart {
    pub fn new(start_data: RouteStartData, fahrplan_eintraege: Vec<FahrplanEintrag>) -> Self {
        Self {
            start_data,
            fahrplan_eintraege,
            has_time_fix: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteStartData {
    pub aufgleis_fahrstrasse: String,
    pub standort_modus: StandortModus,
    pub start_vorschubweg: f32,
    pub speed_anfang: f32,
}

pub fn apply_resolved_route_to_zug(route: ResolvedRoute, zug: &mut Zug) {
    zug.fahrstrassen_name = route.start_data.aufgleis_fahrstrasse;
    zug.standort_modus = route.start_data.standort_modus;
    zug.start_vorschubweg = route.start_data.start_vorschubweg;
    zug.speed_anfang = route.start_data.speed_anfang;
    zug.fahrplan_eintraege = route.fahrplan_eintraege;
}