//! Demo animals shown on first launch, dated relative to `now` so deadlines look realistic.
use crate::model::*;
use chrono::{Duration, NaiveDateTime};

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

/// Three farm animals in different states: Clover (goat, all records done), Jasper (donkey,
/// out of state, still missing a Coggins and ownership paperwork) and Puddles (duck, adopted
/// out). `photos` are relative asset URLs, in that order.
pub fn sample_animals(now: NaiveDateTime, photos: &[Option<String>; 3]) -> Vec<Animal> {
    let days = |n: i64| now - Duration::days(n);
    let health = |ago: i64, kind, desc: &str| HealthEntry {
        date: days(ago).date(),
        kind,
        description: desc.to_string(),
        reason: None,
        result: None,
        rabies_certificate: None,
    };
    let person = |name: &str, addr: &str, dl: &str| {
        Some(Source::Person { name: name.into(), address: addr.into(), drivers_license: dl.into() })
    };

    let clover = Animal {
        id: "sample-clover".into(),
        name: "Clover".into(),
        species: Species::Goat,
        species_other: None,
        sex: Sex::Female,
        received_at: days(21),
        source: person("Dana Whitfield", "412 Elm St, Lincoln, NE 68502", ""),
        out_of_state: false,
        health_certificate: None,
        ownership_doc: s("Surrender form SF-114"),
        coggins: None,
        scrapie_tag: s("NE-4471"),
        identification: Identification { tag: s("Ear tag 12"), ..Default::default() },
        health: vec![
            health(20, HealthKind::Vaccination, "CD&T vaccine"),
            health(14, HealthKind::Treatment, "Hoof trim and dewormer"),
        ],
        story: Story {
            breed: s("Nubian mix"),
            age_text: s("~3 years"),
            color_markings: s("White with tan ears"),
            weight_lbs: Some(110.0),
            altered: YesNoUnknown::Yes,
            horns: Some(Horns::Disbudded),
            housing: Some(Housing::Outdoor),
            good_with_kids: YesNoUnknown::Yes,
            good_with_other_animals: YesNoUnknown::Yes,
            energy: Some(Energy::Medium),
            personality: s("A curious, food-motivated sweetheart who follows you around the pasture."),
            adoption_fee: Some(75),
            ..Default::default()
        },
        photo: photos[0].clone(),
        status: Status::Available,
        status_changed_at: days(7),
        disposition: None,
    };

    // Came from out of state with a CVI, but no Coggins or ownership paperwork yet: two reminders.
    let jasper = Animal {
        id: "sample-jasper".into(),
        name: "Jasper".into(),
        species: Species::Donkey,
        species_other: None,
        sex: Sex::Male,
        received_at: days(3),
        source: Some(Source::Agency {
            agency: "Dawes County Sheriff".into(),
            official_title: "Deputy".into(),
            address: "451 Main St, Chadron, NE 69337".into(),
        }),
        out_of_state: true,
        health_certificate: s("CVI-SD-20931"),
        ownership_doc: None,
        coggins: None,
        scrapie_tag: None,
        identification: Identification::default(),
        health: vec![health(2, HealthKind::Procedure, "Hoof trim")],
        story: Story {
            breed: s("Miniature donkey"),
            age_text: s("~8 years"),
            color_markings: s("Gray with a dark stripe down the back"),
            height_hands: Some(10.2),
            rideable: YesNoUnknown::No,
            altered: YesNoUnknown::Yes,
            good_with_other_animals: YesNoUnknown::Yes,
            energy: Some(Energy::Low),
            personality: s("Shy at first, then a total hugger with a very loud hello."),
            ..Default::default()
        },
        photo: photos[1].clone(),
        status: Status::VetCheck,
        status_changed_at: days(2),
        disposition: None,
    };

    let puddles = Animal {
        id: "sample-puddles".into(),
        name: "Puddles".into(),
        species: Species::Duck,
        species_other: None,
        sex: Sex::Female,
        received_at: days(30),
        source: person("Marcus Bell", "88 Oak Ave, Omaha, NE 68104", ""),
        out_of_state: false,
        health_certificate: None,
        ownership_doc: s("Surrender form SF-097"),
        coggins: None,
        scrapie_tag: None,
        identification: Identification { cage_card: s("Pen 2"), ..Default::default() },
        health: vec![health(29, HealthKind::Treatment, "Wing check, healthy")],
        story: Story {
            breed: s("Pekin"),
            age_text: s("~1 year"),
            color_markings: s("White with an orange bill"),
            layer: YesNoUnknown::Yes,
            housing: Some(Housing::Outdoor),
            good_with_other_animals: YesNoUnknown::Yes,
            personality: s("Loves a puddle, a kiddie pool, and a good muddy waddle."),
            adoption_fee: Some(15),
            ..Default::default()
        },
        photo: photos[2].clone(),
        status: Status::Outcome,
        status_changed_at: days(2),
        disposition: Some(Disposition {
            date: Some(days(2).date()),
            method: OutcomeMethod::Adopted,
            recipient_name: s("Priya Raman"),
            recipient_address: s("905 Cedar Ln, Lincoln, NE 68510"),
            circumstances: None,
            euthanasia_by: None,
            euthanasia_method: None,
            transfer_doc: s("Adoption contract AC-12"),
        }),
    };

    vec![clover, jasper, puddles]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::{check, ItemState};
    use chrono::NaiveDate;

    #[test]
    fn samples_farm_states() {
        let now = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap().and_hms_opt(9, 0, 0).unwrap();
        let photos = [
            Some("assets/samples/clover.svg".to_string()),
            Some("assets/samples/jasper.svg".to_string()),
            Some("assets/samples/puddles.svg".to_string()),
        ];
        let animals = sample_animals(now, &photos);
        let species: Vec<Species> = animals.iter().map(|a| a.species).collect();
        assert_eq!(species, vec![Species::Goat, Species::Donkey, Species::Duck]);
        let statuses: Vec<Status> = animals.iter().map(|a| a.status).collect();
        assert_eq!(statuses, vec![Status::Available, Status::VetCheck, Status::Outcome]);
        assert_eq!(animals[0].photo.as_deref(), Some("assets/samples/clover.svg"));

        let pending = |a: &Animal| {
            check(a, now)
                .into_iter()
                .filter(|i| !matches!(i.state, ItemState::Done | ItemState::NotApplicable))
                .map(|i| i.key)
                .collect::<Vec<_>>()
        };
        assert!(pending(&animals[0]).is_empty(), "{:?}", pending(&animals[0]));
        assert!(pending(&animals[2]).is_empty(), "{:?}", pending(&animals[2]));
        assert_eq!(pending(&animals[1]), vec!["ownership_doc", "coggins"]);
        let jasper = check(&animals[1], now);
        assert!(jasper.iter().any(|i| i.key == "cvi" && i.state == ItemState::Done));
    }
}
