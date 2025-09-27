use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::vec::IntoIter;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_name::FahrplanName;
use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::FahrplanZeile;

#[derive(Debug, PartialEq)]
pub struct RglGglFahrplanZeilen<'a> {
    rgl_ggl: BTreeSet<i32>,
    zeilen: Vec<&'a mut FahrplanZeile>,
}

impl<'a> RglGglFahrplanZeilen<'a> {
    pub fn new(zeile: &'a mut FahrplanZeile) -> Self {
        let mut rgl_ggl = BTreeSet::new();
        rgl_ggl.insert(zeile.fahrplan_regelgleis_gegengleis);
        assert_eq!(rgl_ggl.len(), 1);

        Self {
            rgl_ggl,
            zeilen: vec![zeile],
        }
    }
    
    pub fn first(&self) -> &FahrplanZeile {
        self.zeilen.first().unwrap() // RglGglFahrplanZeilen can only be constructed with at least one entry
    }

    pub fn insert_or_return(&mut self, new_zeile: &'a mut FahrplanZeile) -> Option<&'a mut FahrplanZeile> {
        let zeile = self.zeilen.first().unwrap(); // RglGglFahrplanZeilen can only be constructed with at least one entry

        let text_equals = match (zeile, &new_zeile) {
            (
                FahrplanZeile { fahrplan_name: Some(FahrplanName { fahrplan_name_text, .. }), ..},
                FahrplanZeile { fahrplan_name: Some(FahrplanName { fahrplan_name_text: new_fahrplan_name_text, .. }), ..},
            ) => fahrplan_name_text == new_fahrplan_name_text,
            _ => false,
        };

        if text_equals &&
            zeile.fahrplan_ankunft == new_zeile.fahrplan_ankunft &&
            zeile.fahrplan_abfahrt == new_zeile.fahrplan_abfahrt {
            // insert to set must be executed after other conditions are already checked, otherwise this could lead to an inconsistent state (rgl_ggl inserted but zeile not pushed)
            if self.rgl_ggl.insert(new_zeile.fahrplan_regelgleis_gegengleis) {
                self.zeilen.push(new_zeile);
                None
            } else {
                Some(new_zeile)
            }
        } else {
            Some(new_zeile)
        }
    }

    pub fn into_owned(self) -> OwnedRglGglFahrplanZeilen {
        OwnedRglGglFahrplanZeilen {
            rgl_ggl: self.rgl_ggl,
            zeilen: self.zeilen.into_iter().map(|zeile| zeile.to_owned()).collect(),
        }
    }
}

impl<'a> IntoIterator for RglGglFahrplanZeilen<'a> {
    type Item = &'a mut FahrplanZeile;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.zeilen.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnedRglGglFahrplanZeilen {
    rgl_ggl: BTreeSet<i32>,
    zeilen: Vec<FahrplanZeile>,
}

#[cfg(test)]
impl OwnedRglGglFahrplanZeilen {
    pub fn new(rgl_ggl: BTreeSet<i32>, zeilen: Vec<FahrplanZeile>) -> Self {
        Self {
            rgl_ggl,
            zeilen,
        }
    }
}

impl Display for OwnedRglGglFahrplanZeilen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let first = self.zeilen.first().unwrap(); // RglGglFahrplanZeilen can only be constructed with at least one entry
        write!(
            f,
            "{} [{}]: {:?} - {:?}",
            first.fahrplan_name.as_ref().map(|name| &name.fahrplan_name_text).unwrap_or(&String::from("")),
            self.rgl_ggl.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "),
            first.fahrplan_ankunft.as_ref().map(|ankunft| ankunft.ankunft),
            first.fahrplan_abfahrt.as_ref().map(|abfahrt| abfahrt.abfahrt),
        )
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_abfahrt::FahrplanAbfahrt;
    use zusi_xml_lib::xml::zusi::buchfahrplan::fahrplan_zeile::fahrplan_ankunft::FahrplanAnkunft;
    use super::*;

    #[test]
    fn test_new() {
        let mut zeile = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(0)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("Name".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:39:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:39:57)).build()))
            .build();

        assert_eq!(
            RglGglFahrplanZeilen::new(&mut zeile).into_owned(),
            OwnedRglGglFahrplanZeilen {
                rgl_ggl: [0].into(),
                zeilen: vec![zeile],
            },
        );
    }

    #[test]
    fn test_insert_or_return() {
        let mut zeile_0_a_1 = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(0)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("a".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:39:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:39:57)).build()))
            .build();
        let mut zeile_0_a_2 = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(0)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("a".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:49:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:49:57)).build()))
            .build();
        let mut zeile_1_a_1 = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(1)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("a".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:39:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:39:57)).build()))
            .build();
        let mut zeile_1_a_1_eq = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(1)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("a".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:39:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:39:57)).build()))
            .build();
        let mut zeile_2_b = FahrplanZeile::builder()
            .fahrplan_regelgleis_gegengleis(2)
            .fahrplan_name(Some(FahrplanName::builder().fahrplan_name_text("b".into()).build()))
            .fahrplan_ankunft(Some(FahrplanAnkunft::builder().ankunft(datetime!(2030-02-07 12:39:37)).build()))
            .fahrplan_abfahrt(Some(FahrplanAbfahrt::builder().abfahrt(datetime!(2030-02-07 12:39:57)).build()))
            .build();

        let mut rgl_ggl_fahrplan_zeilen = RglGglFahrplanZeilen::new(&mut zeile_0_a_1);

        assert!(rgl_ggl_fahrplan_zeilen.insert_or_return(&mut zeile_0_a_2).is_some());
        assert_eq!(rgl_ggl_fahrplan_zeilen.insert_or_return(&mut zeile_1_a_1), None);
        assert!(rgl_ggl_fahrplan_zeilen.insert_or_return(&mut zeile_1_a_1_eq).is_some());
        assert!(rgl_ggl_fahrplan_zeilen.insert_or_return(&mut zeile_2_b).is_some());

        assert_eq!(
            rgl_ggl_fahrplan_zeilen.into_owned(),
            OwnedRglGglFahrplanZeilen {
                rgl_ggl: [0, 1].into(),
                zeilen: vec![zeile_0_a_1, zeile_1_a_1],
            },
        );
    }
}