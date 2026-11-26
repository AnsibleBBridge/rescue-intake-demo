//! Demo animals shown on first launch, dated relative to `now` so deadlines look realistic.
use crate::model::*;
use chrono::{Duration, NaiveDateTime};

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

/// Three animals in different states: Biscuit (all records done), Pepper (overdue items),
/// Moose (adopted). `photos` are relative asset URLs, in that order.
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
    let chip = |n: &str| Microchip { number: n.into(), manufacturer: s("HomeAgain"), location: s("Between shoulder blades") };

    let biscuit = Animal {
        id: "sample-biscuit".into(),
        name: "Biscuit".into(),
        species: Species::Dog,
        sex: Sex::Male,
        received_at: days(10),
        source: person("Dana Whitfield", "412 Elm St, Lincoln, NE 68502", "N12345678"),
        out_of_state: false,
        health_certificate: None,
        identification: Identification {
            cage_card: s("K-14"),
            microchip: Some(chip("985112003456789")),
            ..Default::default()
        },
        health: vec![
            HealthEntry { rabies_certificate: s("RC-2291"), ..health(9, HealthKind::RabiesVaccination, "Rabies vaccine, 1 year") },
            health(9, HealthKind::Vaccination, "DHPP booster"),
            health(8, HealthKind::Procedure, "Neutered"),
        ],
        story: Story {
            breed: s("Lab mix"),
            age_text: s("~2 years"),
            color_markings: s("Golden with a white chest"),
            weight_lbs: Some(58.0),
            altered: YesNoUnknown::Yes,
            good_with_dogs: YesNoUnknown::Yes,
            good_with_kids: YesNoUnknown::Yes,
            house_trained: YesNoUnknown::Yes,
            energy: Some(Energy::Medium),
            personality: s("A goofy, food-motivated sweetheart who leans on your leg for pets."),
            adoption_fee: Some(150),
            ..Default::default()
        },
        photo: photos[0].clone(),
        status: Status::Available,
        status_changed_at: days(4),
        disposition: None,
    };

    // Missing driver's license on purpose: past the 48h window it shows Overdue (rabies is only a reminder).
    let pepper = Animal {
        id: "sample-pepper".into(),
        name: "Pepper".into(),
        species: Species::Cat,
        sex: Sex::Female,
        received_at: days(3),
        source: person("Marcus Bell", "88 Oak Ave, Omaha, NE 68104", ""),
        out_of_state: false,
        health_certificate: None,
        identification: Identification { cage_card: s("C-03"), ..Default::default() },
        health: vec![health(3, HealthKind::Treatment, "Flea treatment")],
        story: Story {
            breed: s("Domestic shorthair"),
            age_text: s("~1 year"),
            color_markings: s("Gray tabby"),
            altered: YesNoUnknown::Unknown,
            good_with_cats: YesNoUnknown::Yes,
            energy: Some(Energy::High),
            personality: s("Curious and playful, a little shy for the first few minutes."),
            ..Default::default()
        },
        photo: photos[1].clone(),
        status: Status::VetCheck,
        status_changed_at: days(2),
        disposition: None,
    };

    let moose = Animal {
        id: "sample-moose".into(),
        name: "Moose".into(),
        species: Species::Dog,
        sex: Sex::Male,
        received_at: days(30),
        source: Some(Source::Agency {
            agency: "Lancaster County Animal Control".into(),
            official_title: "Animal Control Officer".into(),
            address: "3140 N 35th St, Lincoln, NE 68504".into(),
        }),
        out_of_state: false,
        health_certificate: None,
        identification: Identification { tag: s("T-5521"), microchip: Some(chip("985112009876543")), ..Default::default() },
        health: vec![
            HealthEntry { rabies_certificate: s("RC-2105"), ..health(29, HealthKind::RabiesVaccination, "Rabies vaccine, 3 year") },
            health(28, HealthKind::Procedure, "Neutered"),
        ],
        story: Story {
            breed: s("Great Pyrenees mix"),
            age_text: s("~4 years"),
            altered: YesNoUnknown::Yes,
            good_with_dogs: YesNoUnknown::Yes,
            good_with_cats: YesNoUnknown::Yes,
            good_with_kids: YesNoUnknown::Yes,
            energy: Some(Energy::Low),
            personality: s("A gentle giant who loves lazy afternoons."),
            adoption_fee: Some(200),
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
        }),
    };

    vec![biscuit, pepper, moose]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::{check, ItemState};
    use chrono::NaiveDate;

    #[test]
    fn samples_cover_three_states() {
        let now = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap().and_hms_opt(9, 0, 0).unwrap();
        let photos = [
            Some("assets/samples/biscuit.svg".to_string()),
            Some("assets/samples/pepper.svg".to_string()),
            Some("assets/samples/moose.svg".to_string()),
        ];
        let animals = sample_animals(now, &photos);
        let statuses: Vec<Status> = animals.iter().map(|a| a.status).collect();
        assert_eq!(statuses, vec![Status::Available, Status::VetCheck, Status::Outcome]);
        assert_eq!(animals[0].photo.as_deref(), Some("assets/samples/biscuit.svg"));

        let pepper = check(&animals[1], now);
        assert!(pepper.iter().any(|i| matches!(i.state, ItemState::Overdue { .. })));
        // Biscuit and Moose are fully documented: nothing pending.
        for a in [&animals[0], &animals[2]] {
            let bad = check(a, now)
                .into_iter()
                .filter(|i| matches!(i.state, ItemState::Open { .. } | ItemState::Overdue { .. }))
                .map(|i| i.key)
                .collect::<Vec<_>>();
            assert!(bad.is_empty(), "{}: {bad:?}", a.name);
        }
    }
}
