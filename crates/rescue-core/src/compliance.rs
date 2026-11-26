//! Nebraska (23 NAC 18) record-keeping checklist for one animal.
use crate::model::*;
use chrono::{Duration, NaiveDateTime};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemState {
    Done,
    Open { due: NaiveDateTime },
    Overdue { due: NaiveDateTime },
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComplianceItem {
    pub key: &'static str,
    pub label: &'static str,
    pub section: &'static str,
    pub state: ItemState,
}

/// Blank means empty or whitespace-only.
fn filled(s: &str) -> bool {
    !s.trim().is_empty()
}

fn filled_opt(s: &Option<String>) -> bool {
    s.as_deref().is_some_and(filled)
}

fn source_done(source: &Option<Source>) -> bool {
    match source {
        Some(Source::Person { name, address, drivers_license }) => {
            filled(name) && filled(address) && filled(drivers_license)
        }
        Some(Source::Agency { agency, official_title, address }) => {
            filled(agency) && filled(official_title) && filled(address)
        }
        None => false,
    }
}

fn disposition_done(d: &Disposition) -> bool {
    let has_date = d.date.is_some();
    match d.method {
        OutcomeMethod::Adopted | OutcomeMethod::Transferred | OutcomeMethod::ReturnedToOwner => {
            has_date && filled_opt(&d.recipient_name) && filled_opt(&d.recipient_address)
        }
        OutcomeMethod::Died => has_date && filled_opt(&d.circumstances),
        OutcomeMethod::Euthanized => {
            has_date
                && filled_opt(&d.circumstances)
                && filled_opt(&d.euthanasia_by)
                && filled_opt(&d.euthanasia_method)
        }
    }
}

/// Evaluate every checklist item; deadlines are 48h after intake (or after the outcome).
pub fn check(animal: &Animal, now: NaiveDateTime) -> Vec<ComplianceItem> {
    let intake_due = animal.received_at + Duration::hours(48);
    // Pending items are Open until the deadline passes, then Overdue.
    let state = |done: bool, due: NaiveDateTime| {
        if done {
            ItemState::Done
        } else if now > due {
            ItemState::Overdue { due }
        } else {
            ItemState::Open { due }
        }
    };
    let na_or = |applies: bool, done: bool, due: NaiveDateTime| {
        if applies { state(done, due) } else { ItemState::NotApplicable }
    };

    let chip = animal.identification.microchip.as_ref();
    let chip_done = chip.is_some_and(|c| filled_opt(&c.manufacturer) && filled_opt(&c.location));

    let disp_due = animal
        .disposition
        .as_ref()
        .and_then(|d| d.date)
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
        .unwrap_or(animal.status_changed_at)
        + Duration::hours(48);
    let is_outcome = animal.status == Status::Outcome;
    let disp_ok = animal.disposition.as_ref().is_some_and(disposition_done);

    let item = |key, label, section, state| ComplianceItem { key, label, section, state };
    vec![
        item("source", "Source name, address & ID", "23 NAC 18-010.04A(3)",
            state(source_done(&animal.source), intake_due)),
        item("identification", "Individual ID (tag, cage card, tattoo, or chip)",
            "23 NAC 18-008 / 010.04A(4)",
            state(animal.identification.has_any(), intake_due)),
        item("microchip_details", "Microchip manufacturer & location", "23 NAC 18-008.07",
            na_or(chip.is_some(), chip_done, intake_due)),
        item("health_certificate", "Health certificate (imported animals)",
            "23 NAC 18-010.04A(5)",
            na_or(animal.out_of_state, filled_opt(&animal.health_certificate), intake_due)),
        item("health_record", "Health record started", "23 NAC 18-010.06",
            state(!animal.health.is_empty(), intake_due)),
        item("rabies", "Rabies vaccination recorded", "23 NAC 18-010.06A",
            state(animal.health.iter().any(|h| h.kind == HealthKind::RabiesVaccination), intake_due)),
        item("disposition", "Outcome record (date, recipient, method)", "23 NAC 18-010.05A",
            na_or(is_outcome, disp_ok, disp_due)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn dt(d: u32, h: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 11, d).unwrap().and_hms_opt(h, 0, 0).unwrap()
    }

    fn animal() -> Animal {
        Animal {
            id: "a1".into(),
            name: "Test".into(),
            species: Species::Dog,
            sex: Sex::Male,
            received_at: dt(20, 10),
            source: None,
            out_of_state: false,
            health_certificate: None,
            identification: Identification::default(),
            health: vec![],
            story: Story::default(),
            photo: None,
            status: Status::Intake,
            status_changed_at: dt(20, 10),
            disposition: None,
        }
    }

    fn person(dl: &str) -> Source {
        Source::Person { name: "Jo".into(), address: "1 Main St".into(), drivers_license: dl.into() }
    }

    fn state_of(items: &[ComplianceItem], key: &str) -> ItemState {
        items.iter().find(|i| i.key == key).unwrap().state
    }

    #[test]
    fn fresh_intake_items_open_with_48h_due() {
        let items = check(&animal(), dt(20, 12));
        assert_eq!(state_of(&items, "source"), ItemState::Open { due: dt(22, 10) });
    }

    #[test]
    fn past_48h_is_overdue() {
        let items = check(&animal(), dt(23, 12));
        assert_eq!(state_of(&items, "source"), ItemState::Overdue { due: dt(22, 10) });
    }

    #[test]
    fn complete_person_source_is_done() {
        let mut a = animal();
        a.source = Some(person("N123"));
        assert_eq!(state_of(&check(&a, dt(20, 12)), "source"), ItemState::Done);
    }

    #[test]
    fn blank_dl_not_done() {
        let mut a = animal();
        a.source = Some(person("   "));
        assert!(matches!(state_of(&check(&a, dt(20, 12)), "source"), ItemState::Open { .. }));
    }

    #[test]
    fn microchip_details_na_without_chip() {
        let items = check(&animal(), dt(20, 12));
        assert_eq!(state_of(&items, "microchip_details"), ItemState::NotApplicable);
    }

    #[test]
    fn health_cert_only_when_out_of_state() {
        let mut a = animal();
        assert_eq!(state_of(&check(&a, dt(20, 12)), "health_certificate"), ItemState::NotApplicable);
        a.out_of_state = true;
        assert!(matches!(state_of(&check(&a, dt(20, 12)), "health_certificate"), ItemState::Open { .. }));
        a.health_certificate = Some("CVI-77".into());
        assert_eq!(state_of(&check(&a, dt(20, 12)), "health_certificate"), ItemState::Done);
    }

    #[test]
    fn euthanized_requires_by_and_method() {
        let mut a = animal();
        a.status = Status::Outcome;
        let mut d = Disposition {
            date: Some(NaiveDate::from_ymd_opt(2026, 11, 21).unwrap()),
            method: OutcomeMethod::Euthanized,
            recipient_name: None,
            recipient_address: None,
            circumstances: Some("Severe injury".into()),
            euthanasia_by: None,
            euthanasia_method: None,
        };
        a.disposition = Some(d.clone());
        assert!(matches!(state_of(&check(&a, dt(21, 12)), "disposition"), ItemState::Open { .. }));
        d.euthanasia_by = Some("Dr. Lee".into());
        d.euthanasia_method = Some("Sodium pentobarbital".into());
        a.disposition = Some(d);
        assert_eq!(state_of(&check(&a, dt(21, 12)), "disposition"), ItemState::Done);
    }

    #[test]
    fn disposition_na_before_outcome() {
        let items = check(&animal(), dt(20, 12));
        assert_eq!(state_of(&items, "disposition"), ItemState::NotApplicable);
    }
}
