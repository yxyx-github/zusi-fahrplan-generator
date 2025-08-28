use crate::core::schedule::prepare_entries::prepare_entries;
use crate::input::schedule::Schedule;
use zusi_xml_lib::xml::zusi::zug::fahrplan_eintrag::FahrplanEintrag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyScheduleError {

}

pub fn apply(mut fahrplan_eintraege: &mut Vec<FahrplanEintrag>, schedule: &Schedule) -> Result<(), ApplyScheduleError> {
    prepare_entries(fahrplan_eintraege, schedule).into_iter().for_each(
        |(fahrplan_eintrag, schedule)|
            todo!()
    );

    Ok(())
}