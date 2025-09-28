use crate::core::lib::generated_zug::RawGeneratedZug;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;
use zusi_xml_lib::xml::zusi::zug::standort_modus::StandortModus;
use crate::input::fahrplan_config::StartFahrzeugVerbandAktion;

// TODO: refactor using builder?
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoute {
    pub start_data: RouteStartData,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
    pub fahrplan_zeilen: Vec<FahrplanZeile>,
    pub mindest_bremshundertstel: f32,
}

impl From<ResolvedRoutePart> for ResolvedRoute {
    fn from(ResolvedRoutePart { start_data, fahrplan_eintraege, fahrplan_zeilen, mindest_bremshundertstel, .. }: ResolvedRoutePart) -> Self {
        Self {
            start_data,
            fahrplan_eintraege,
            fahrplan_zeilen,
            mindest_bremshundertstel,
        }
    }
}

// TODO: refactor using builder?
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRoutePart {
    pub start_data: RouteStartData,
    pub fahrplan_eintraege: Vec<FahrplanEintrag>,
    pub has_time_fix: bool,
    pub fahrplan_zeilen: Vec<FahrplanZeile>,
    pub mindest_bremshundertstel: f32,
}

impl ResolvedRoutePart {
    pub fn new(start_data: RouteStartData, fahrplan_eintraege: Vec<FahrplanEintrag>, fahrplan_zeilen: Vec<FahrplanZeile>, mindest_bremshundertstel: f32) -> Self {
        Self {
            start_data,
            fahrplan_eintraege,
            has_time_fix: false,
            fahrplan_zeilen,
            mindest_bremshundertstel,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteStartData {
    pub aufgleis_fahrstrasse: String,
    pub standort_modus: StandortModus,
    pub start_vorschubweg: f32,
    pub speed_anfang: f32,
    pub km_start: Option<f32>,
    pub gnt_spalte: Option<bool>,
    pub fahrzeug_verband_aktion: Option<StartFahrzeugVerbandAktion>,
}

pub fn apply_resolved_route_to_zug(route: ResolvedRoute, zug: &mut RawGeneratedZug) {
    zug.zug.fahrstrassen_name = route.start_data.aufgleis_fahrstrasse;
    zug.zug.standort_modus = route.start_data.standort_modus;
    zug.zug.start_vorschubweg = route.start_data.start_vorschubweg;
    zug.zug.speed_anfang = route.start_data.speed_anfang;
    zug.zug.fahrplan_eintraege = route.fahrplan_eintraege;
    if let Some(buchfahrplan) = &mut zug.buchfahrplan {
        buchfahrplan.fahrplan_zeilen = route.fahrplan_zeilen;
    }
}