use zusi_xml_lib::xml::zusi::buchfahrplan::Buchfahrplan;
use zusi_xml_lib::xml::zusi::info::{DateiTyp, Info};
use zusi_xml_lib::xml::zusi::zug::Zug;
use zusi_xml_lib::xml::zusi::TypedZusi;

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedZug {
    pub zug: TypedZusi<Zug>,
    pub buchfahrplan: Option<TypedZusi<Buchfahrplan>>,
}

impl From<RawGeneratedZug> for GeneratedZug {
    fn from(raw: RawGeneratedZug) -> Self {
        Self {
            zug: TypedZusi::<Zug>::builder()
                .info(Info::builder().datei_typ(DateiTyp::Zug).version("A.6".into()).min_version("A.6".into()).build())
                .value(raw.zug)
                .build(),
            buchfahrplan: raw.buchfahrplan.map(|buchfahrplan|
                TypedZusi::<Buchfahrplan>::builder()
                    .info(Info::builder().datei_typ(DateiTyp::Buchfahrplan).version("A.4".into()).min_version("A.4".into()).build())
                    .value(buchfahrplan)
                    .build()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawGeneratedZug {
    pub zug: Zug,
    pub buchfahrplan: Option<Buchfahrplan>,
}

impl From<(Zug, Option<Buchfahrplan>)> for RawGeneratedZug {
    fn from((zug, buchfahrplan): (Zug, Option<Buchfahrplan>)) -> Self {
        Self {
            zug,
            buchfahrplan,
        }
    }
}