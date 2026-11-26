//! Versioned top-level storage blob.
use crate::model::*;
use serde::{Deserialize, Serialize};

/// Bump when the stored shape changes incompatibly.
pub const SCHEMA_VERSION: u32 = 1;

/// Everything persisted in the browser, as one versioned JSON blob.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoreData {
    pub version: u32,
    pub animals: Vec<Animal>,
    pub settings: Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StoreError {
    /// Not parseable as StoreData.
    Corrupt(String),
    /// Written by a different schema version; refuse rather than misread.
    WrongVersion(u32),
}

impl StoreData {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("StoreData is always serializable")
    }

    pub fn from_json(s: &str) -> Result<StoreData, StoreError> {
        let data: StoreData = serde_json::from_str(s).map_err(|e| StoreError::Corrupt(e.to_string()))?;
        if data.version != SCHEMA_VERSION {
            return Err(StoreError::WrongVersion(data.version));
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    fn dt(d: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 3, d).unwrap().and_hms_opt(9, 30, 0).unwrap()
    }

    fn sample() -> StoreData {
        let animal = Animal {
            id: "a1".into(),
            name: "Biscuit".into(),
            species: Species::Dog,
            sex: Sex::Male,
            received_at: dt(1),
            source: Some(Source::Person {
                name: "Pat Doe".into(),
                address: "1 Main St, Omaha, NE".into(),
                drivers_license: "A1234567".into(),
            }),
            out_of_state: false,
            health_certificate: None,
            identification: Identification {
                tag: None,
                cage_card: Some("C-4".into()),
                tattoo: None,
                microchip: Some(Microchip {
                    number: "985112345678901".into(),
                    manufacturer: Some("HomeAgain".into()),
                    location: Some("between shoulders".into()),
                }),
            },
            health: vec![HealthEntry {
                date: NaiveDate::from_ymd_opt(2026, 3, 2).unwrap(),
                kind: HealthKind::RabiesVaccination,
                description: "Rabies 1yr".into(),
                reason: None,
                result: None,
                rabies_certificate: Some("R-100".into()),
            }],
            story: Story::default(),
            photo: None,
            status: Status::Intake,
            status_changed_at: dt(1),
            disposition: None,
        };
        StoreData { version: SCHEMA_VERSION, animals: vec![animal], settings: Settings::default() }
    }

    #[test]
    fn round_trip_preserves_animals() {
        let data = sample();
        assert_eq!(StoreData::from_json(&data.to_json()).unwrap(), data);
    }

    #[test]
    fn corrupt_json_is_error() {
        assert!(matches!(StoreData::from_json("{not json"), Err(StoreError::Corrupt(_))));
    }

    #[test]
    fn future_version_is_error() {
        let mut d = sample();
        d.version = 99;
        assert_eq!(StoreData::from_json(&d.to_json()), Err(StoreError::WrongVersion(99)));
    }

    #[test]
    fn identification_has_any() {
        let mut id = Identification::default();
        assert!(!id.has_any());
        id.tag = Some("T1".into());
        assert!(id.has_any());
    }
}
