use std::num::{ParseIntError, TryFromIntError};

const MULTI_NUMMER_SEPARATOR: &str = "_";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZugNummer(Vec<u32>);

impl ZugNummer {
    fn new(value: Vec<u32>) -> Self {
        assert!(!value.is_empty()); // Int cannot be parsed from empty string, empty Vec should be impossible
        Self(value)
    }

    pub fn increment(&mut self, increment: i32) -> Result<(), TryFromIntError> {
        self.0.iter_mut().try_for_each(|nummer| {
            let new = *nummer as i32 + increment;
            *nummer = new.try_into()?;
            Ok(())
        })
    }

    pub fn from_str(value: &str) -> Result<Self, ParseIntError> {
        Ok(Self::new(
            value
                .split(MULTI_NUMMER_SEPARATOR)
                .try_fold(
                    vec![],
                    |mut acc, str_part| {
                        acc.push(str_part.parse()?);
                        Ok(acc)
                    },
                )?
        ))
    }

    pub fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|nummer| nummer.to_string())
            .collect::<Vec<_>>()
            .join(MULTI_NUMMER_SEPARATOR)
    }

    pub fn to_new_incremented(&self, increment: i32) -> Result<Self, TryFromIntError> {
        let mut new = self.clone();
        new.increment(increment)?;
        Ok(new)
    }
}

impl TryFrom<&str> for ZugNummer {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ZugNummer::from_str(value)
    }
}

impl TryFrom<&String> for ZugNummer {
    type Error = ParseIntError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        ZugNummer::from_str(value)
    }
}

impl TryFrom<String> for ZugNummer {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ZugNummer::from_str(&value)
    }
}

impl From<ZugNummer> for String {
    fn from(value: ZugNummer) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zug_nummer_from_string() {
        assert_eq!(
            ZugNummer::try_from("03").unwrap(),
            ZugNummer(vec![3]),
        );
        assert_eq!(
            ZugNummer::try_from("03_70_22").unwrap(),
            ZugNummer(vec![3, 70, 22]),
        );
        assert!(matches!(
            ZugNummer::try_from("4__3").unwrap_err(),
            ParseIntError { .. },
        ));
        assert!(matches!(
            ZugNummer::try_from("").unwrap_err(),
            ParseIntError { .. },
        ));
    }

    #[test]
    fn test_zug_nummer_to_string() {
        assert_eq!(
            ZugNummer(vec![3, 70, 22]).to_string(),
            String::from("3_70_22"),
        );
        assert_eq!(
            ZugNummer(vec![]).to_string(), // impossible case, see ZugNummer::new()
            String::from(""),
        );
    }

    #[test]
    fn test_increment_zug_nummer() {
        assert_eq!(
            ZugNummer::new(vec![1, 2, 3, 100, 1000]).to_new_incremented(3).unwrap(),
            ZugNummer::new(vec![4, 5, 6, 103, 1003]),
        );
        assert_eq!(
            ZugNummer::new(vec![3, 100, 1000]).to_new_incremented(-3).unwrap(),
            ZugNummer::new(vec![0, 97, 997]),
        );
        let _: TryFromIntError = ZugNummer::new(vec![1, 2, 3, 100, 1000]).to_new_incremented(-3).unwrap_err();
    }

    /// impossible case, see ZugNummer::new()
    #[test]
    #[should_panic]
    fn test_increment_empty_zug_nummer() {
        let _ = ZugNummer::new(vec![]).to_new_incremented(-3);
    }
}