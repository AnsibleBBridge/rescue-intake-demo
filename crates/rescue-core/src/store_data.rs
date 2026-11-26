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
            species_other: None,
            sex: Sex::Male,
            received_at: dt(1),
            source: Some(Source::Person {
                name: "Pat Doe".into(),
                address: "1 Main St, Omaha, NE".into(),
                drivers_license: "A1234567".into(),
            }),
            out_of_state: false,
            health_certificate: None,
            ownership_doc: None,
            coggins: None,
            scrapie_tag: None,
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

    #[test]
    fn v1_blob_still_loads() {
        // Literal v1 shape: no species_other / coggins / scrapie / transfer_doc / new story fields.
        let v1 = r##"{"version":1,"animals":[{"id":"a1","name":"Rex","species":"Dog","sex":"Male",
            "received_at":"2026-03-01T09:30:00","source":null,"out_of_state":false,
            "health_certificate":null,"identification":{"tag":null,"cage_card":null,"tattoo":null,"microchip":null},
            "health":[],"story":{"breed":null,"age_text":null,"birth_date":null,"color_markings":null,
            "weight_lbs":null,"altered":"Unknown","good_with_dogs":"Unknown","good_with_cats":"Unknown",
            "good_with_kids":"Unknown","house_trained":"Unknown","energy":null,"personality":null,
            "special_needs":null,"adoption_fee":null},"photo":null,"status":"Outcome",
            "status_changed_at":"2026-03-02T09:30:00",
            "disposition":{"date":null,"method":"Adopted","recipient_name":null,"recipient_address":null,
            "circumstances":null,"euthanasia_by":null,"euthanasia_method":null}}],
            "settings":{"rescue_name":"R","phone":"1","email":"e","website":"","hashtags":["#AdoptDontShop"]}}"##;
        let d = StoreData::from_json(v1).unwrap();
        let a = &d.animals[0];
        assert_eq!(a.species, Species::Dog);
        assert!(a.species_other.is_none() && a.coggins.is_none() && a.scrapie_tag.is_none() && a.ownership_doc.is_none());
        assert!(a.story.horns.is_none() && a.story.housing.is_none() && a.story.height_hands.is_none());
        assert_eq!(a.story.rideable, YesNoUnknown::Unknown);
        assert_eq!(a.story.layer, YesNoUnknown::Unknown);
        assert_eq!(a.story.good_with_other_animals, YesNoUnknown::Unknown);
        assert!(a.disposition.as_ref().unwrap().transfer_doc.is_none());
    }

    #[test]
    fn round_trip_with_v2_fields() {
        let mut data = sample();
        let a = &mut data.animals[0];
        a.species = Species::Goat;
        a.scrapie_tag = Some("NE-123".into());
        a.coggins = Some(Coggins {
            date: NaiveDate::from_ymd_opt(2026, 2, 1),
            lab: Some("Lab A".into()),
            result: Some("Negative".into()),
        });
        a.story.horns = Some(Horns::Polled);
        a.story.housing = Some(Housing::Either);
        a.story.height_hands = Some(14.2);
        a.disposition = Some(Disposition {
            date: None,
            method: OutcomeMethod::Adopted,
            recipient_name: None,
            recipient_address: None,
            circumstances: None,
            euthanasia_by: None,
            euthanasia_method: None,
            transfer_doc: Some("BOS-9".into()),
        });
        assert_eq!(StoreData::from_json(&data.to_json()).unwrap(), data);
    }
}
