const DATE_TIME_FORMAT: &str = "[year]-[month]-[day] [hour]:[minute]:[second]";

pub mod date_time_format {
    use serde::{de, ser, Deserialize, Deserializer, Serializer};
    use time::{format_description, PrimitiveDateTime};
    use crate::serde_helpers::with::DATE_TIME_FORMAT;

    pub fn serialize<S>(pdt: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let format = format_description::parse(DATE_TIME_FORMAT).map_err(ser::Error::custom)?;
        let formatted = pdt.format(&format).map_err(ser::Error::custom)?;
        serializer.serialize_str(&formatted).map_err(ser::Error::custom)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error> where D: Deserializer<'de> {
        let format = format_description::parse(DATE_TIME_FORMAT).map_err(de::Error::custom)?;
        let s = String::deserialize(deserializer)?;
        PrimitiveDateTime::parse(&s, &format).map_err(de::Error::custom)
    }
}

pub mod date_time_option_format {
    use serde::{de, ser, Deserialize, Deserializer, Serializer};
    use time::{format_description, PrimitiveDateTime};
    use crate::serde_helpers::with::DATE_TIME_FORMAT;

    pub fn serialize<S>(pdt: &Option<PrimitiveDateTime>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match pdt {
            None => serializer.serialize_str("").map_err(ser::Error::custom),
            Some(pdt) => {
                let format = format_description::parse(DATE_TIME_FORMAT).map_err(ser::Error::custom)?;
                let formatted = pdt.format(&format).map_err(ser::Error::custom)?;
                serializer.serialize_str(&formatted).map_err(ser::Error::custom)
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PrimitiveDateTime>, D::Error> where D: Deserializer<'de> {
        let value = String::deserialize(deserializer)?;
        if value == "" {
            return Ok(None);
        }
        let format = format_description::parse(DATE_TIME_FORMAT).map_err(de::Error::custom)?;
        Ok(Some(PrimitiveDateTime::parse(&value, &format).map_err(de::Error::custom)?))
    }
}

pub mod duration_format {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::Duration;
    use crate::serde_helpers::{format_duration_as_time_string, parse_duration_from_time_string};

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = format_duration_as_time_string(value);
        serializer.serialize_str(&formatted)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(parse_duration_from_time_string(&s).map_err(serde::de::Error::custom)?)
    }
}

pub mod duration_option_format {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::Duration;
    use crate::serde_helpers::{format_duration_as_time_string, parse_duration_from_time_string};

    pub fn serialize<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            None => {
                serializer.serialize_str("")
            }
            Some(value) => {
                let formatted = format_duration_as_time_string(value);
                serializer.serialize_str(&formatted)
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(
            match s.as_str() {
                "" => None,
                s => Some(parse_duration_from_time_string(&s).map_err(serde::de::Error::custom)?),
            }
        )
    }
}