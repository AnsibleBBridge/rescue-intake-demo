//! Nebraska (23 NAC 18) record-keeping checklist for one animal.
use crate::model::*;
use chrono::{Duration, NaiveDateTime};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemState {
    Done,
    Open { due: NaiveDateTime },
    Overdue { due: NaiveDateTime },
    NotApplicable,
    /// Worth doing but carries no deadline, so it is never Open or Overdue.
    Reminder,
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

/// Evaluate every checklist item. Dogs/cats get the 23 NAC 18 items with 48h deadlines;
/// every other species gets deadline-free reminders (see `check_farm`).
pub fn check(animal: &Animal, now: NaiveDateTime) -> Vec<ComplianceItem> {
    match animal.species.family() {
        Family::Dog | Family::Cat => check_dog_cat(animal, now),
        _ => check_farm(animal),
    }
}

/// Source for non-dog/cat animals: who it came from and where; no driver's license needed.
fn contact_done(source: &Option<Source>) -> bool {
    match source {
        Some(Source::Person { name, address, .. }) => filled(name) && filled(address),
        Some(Source::Agency { agency, address, .. }) => filled(agency) && filled(address),
        None => false,
    }
}

/// Farm/other species: nothing here has a legal 48h clock, so missing items are Reminders.
/// The one exception is the scrapie tag on a sheep/goat that has already left.
fn check_farm(animal: &Animal) -> Vec<ComplianceItem> {
    const BRAND: &str = "Nebraska Brand Committee (proof of ownership)";
    const IMPORT: &str = "NDA import requirements";
    const GOOD: &str = "Good practice";
    let reminder = |done: bool| if done { ItemState::Done } else { ItemState::Reminder };
    let na_or = |applies: bool, done: bool| if applies { reminder(done) } else { ItemState::NotApplicable };
    let item = |key, label, section, state| ComplianceItem { key, label, section, state };

    let disp = animal.disposition.as_ref();
    let is_outcome = animal.status == Status::Outcome;
    let left_to_new_owner = is_outcome
        && disp.is_some_and(|d| matches!(d.method, OutcomeMethod::Adopted | OutcomeMethod::Transferred));

    let mut items = vec![
        item("source_contact", "Where it came from (name & address)", GOOD,
            reminder(contact_done(&animal.source))),
        item("ownership_doc", "Proof of ownership or surrender form", BRAND,
            reminder(filled_opt(&animal.ownership_doc))),
        item("health_record", "Vet / health notes started", GOOD, reminder(!animal.health.is_empty())),
        item("cvi", "Certificate of veterinary inspection (arrived from out of state)", IMPORT,
            na_or(animal.out_of_state, filled_opt(&animal.health_certificate))),
    ];

    match animal.species.family() {
        Family::Equine => items.push(item(
            "coggins",
            "Coggins (EIA) test on file",
            if animal.out_of_state { IMPORT } else { GOOD },
            reminder(animal.coggins.as_ref().is_some_and(|c| c.date.is_some())),
        )),
        Family::SmallRuminant => {
            let has_tag = filled_opt(&animal.scrapie_tag);
            // Once adopted/transferred without a tag the ID rule was already missed: flag it hard.
            let state = if has_tag {
                ItemState::Done
            } else if left_to_new_owner {
                let due = disp
                    .and_then(|d| d.date)
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap_or(animal.status_changed_at);
                ItemState::Overdue { due }
            } else {
                ItemState::Reminder
            };
            items.push(item("scrapie_tag", "Official scrapie tag #", "NDA scrapie ID rules (9 CFR 79)", state));
        }
        _ => {}
    }

    items.push(item("outcome_record", "Outcome record (date, who it went to)", GOOD,
        na_or(is_outcome, disp.is_some_and(disposition_done))));
    items.push(item("transfer_doc", "Bill of sale or adoption contract", BRAND,
        na_or(left_to_new_owner, disp.is_some_and(|d| filled_opt(&d.transfer_doc)))));
    items
}

/// Dog/cat checklist; deadlines are 48h after intake (or after the outcome).
fn check_dog_cat(animal: &Animal, now: NaiveDateTime) -> Vec<ComplianceItem> {
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
        // 010.06A only says health records include the rabies certificate; it sets no deadline.
        item("rabies", "Rabies certificate on file (if vaccinated)", "23 NAC 18-010.06A",
            if animal.health.iter().any(|h| h.kind == HealthKind::RabiesVaccination) {
                ItemState::Done
            } else {
                ItemState::Reminder
            }),
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
            species_other: None,
            sex: Sex::Male,
            received_at: dt(20, 10),
            source: None,
            out_of_state: false,
            health_certificate: None,
            ownership_doc: None,
            coggins: None,
            scrapie_tag: None,
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
            transfer_doc: None,
        };
        a.disposition = Some(d.clone());
        assert!(matches!(state_of(&check(&a, dt(21, 12)), "disposition"), ItemState::Open { .. }));
        d.euthanasia_by = Some("Dr. Lee".into());
        d.euthanasia_method = Some("Sodium pentobarbital".into());
        a.disposition = Some(d);
        assert_eq!(state_of(&check(&a, dt(21, 12)), "disposition"), ItemState::Done);
    }

    #[test]
    fn rabies_is_a_reminder_never_overdue() {
        let mut a = animal();
        // Days past intake: still a Reminder, not Overdue.
        assert_eq!(state_of(&check(&a, dt(24, 12)), "rabies"), ItemState::Reminder);
        a.health.push(HealthEntry {
            date: dt(20, 10).date(),
            kind: HealthKind::RabiesVaccination,
            description: "Rabies".into(),
            reason: None,
            result: None,
            rabies_certificate: None,
        });
        assert_eq!(state_of(&check(&a, dt(24, 12)), "rabies"), ItemState::Done);
    }

    #[test]
    fn disposition_na_before_outcome() {
        let items = check(&animal(), dt(20, 12));
        assert_eq!(state_of(&items, "disposition"), ItemState::NotApplicable);
    }

    // ---- farm / other species ----

    fn farm(sp: Species) -> Animal {
        Animal { species: sp, ..animal() }
    }

    fn outcome(a: &mut Animal, m: OutcomeMethod) {
        a.status = Status::Outcome;
        a.disposition = Some(Disposition {
            date: Some(NaiveDate::from_ymd_opt(2026, 11, 22).unwrap()),
            method: m,
            recipient_name: Some("Sam".into()),
            recipient_address: Some("2 Farm Rd".into()),
            circumstances: None,
            euthanasia_by: None,
            euthanasia_method: None,
            transfer_doc: None,
        });
    }

    fn section_of<'a>(items: &'a [ComplianceItem], key: &str) -> &'a str {
        items.iter().find(|i| i.key == key).unwrap().section
    }

    #[test]
    fn dog_items_unchanged() {
        let keys: Vec<_> = check(&animal(), dt(20, 12)).iter().map(|i| i.key).collect();
        assert_eq!(keys, ["source", "identification", "microchip_details", "health_certificate", "health_record", "rabies", "disposition"]);
        let cat = check(&farm(Species::Cat), dt(23, 12));
        assert_eq!(state_of(&cat, "source"), ItemState::Overdue { due: dt(22, 10) });
    }

    #[test]
    fn goat_without_tag_is_reminder() {
        let items = check(&farm(Species::Goat), dt(30, 12));
        assert_eq!(state_of(&items, "scrapie_tag"), ItemState::Reminder);
        assert_eq!(section_of(&items, "scrapie_tag"), "NDA scrapie ID rules (9 CFR 79)");
        let mut a = farm(Species::Sheep);
        a.scrapie_tag = Some("NE 123".into());
        assert_eq!(state_of(&check(&a, dt(30, 12)), "scrapie_tag"), ItemState::Done);
    }

    #[test]
    fn goat_adopted_without_tag_is_overdue() {
        let mut a = farm(Species::Goat);
        outcome(&mut a, OutcomeMethod::Adopted);
        assert_eq!(state_of(&check(&a, dt(23, 12)), "scrapie_tag"), ItemState::Overdue { due: dt(22, 0) });
        // Died is not a transfer: still just a reminder.
        outcome(&mut a, OutcomeMethod::Died);
        assert_eq!(state_of(&check(&a, dt(23, 12)), "scrapie_tag"), ItemState::Reminder);
    }

    #[test]
    fn donkey_coggins_section_depends_on_out_of_state() {
        let mut a = farm(Species::Donkey);
        assert_eq!(section_of(&check(&a, dt(21, 12)), "coggins"), "Good practice");
        a.out_of_state = true;
        let items = check(&a, dt(21, 12));
        assert_eq!(section_of(&items, "coggins"), "NDA import requirements");
        assert_eq!(state_of(&items, "coggins"), ItemState::Reminder);
        a.coggins = Some(Coggins { date: Some(dt(1, 0).date()), lab: None, result: None });
        assert_eq!(state_of(&check(&a, dt(21, 12)), "coggins"), ItemState::Done);
    }

    #[test]
    fn duck_has_no_coggins_or_scrapie_items() {
        let items = check(&farm(Species::Duck), dt(21, 12));
        assert!(items.iter().all(|i| i.key != "coggins" && i.key != "scrapie_tag"));
        assert_eq!(state_of(&items, "source_contact"), ItemState::Reminder);
        assert_eq!(state_of(&items, "ownership_doc"), ItemState::Reminder);
    }

    #[test]
    fn non_dog_items_never_overdue_except_scrapie() {
        let items = check(&farm(Species::Horse), dt(30, 12));
        assert!(items.iter().all(|i| !matches!(i.state, ItemState::Overdue { .. } | ItemState::Open { .. })));
    }

    #[test]
    fn cvi_only_when_out_of_state() {
        let mut a = farm(Species::Goat);
        assert_eq!(state_of(&check(&a, dt(21, 12)), "cvi"), ItemState::NotApplicable);
        a.out_of_state = true;
        assert_eq!(state_of(&check(&a, dt(21, 12)), "cvi"), ItemState::Reminder);
        a.health_certificate = Some("CVI-1".into());
        assert_eq!(state_of(&check(&a, dt(21, 12)), "cvi"), ItemState::Done);
    }

    #[test]
    fn transfer_doc_only_for_adopted_or_transferred() {
        let mut a = farm(Species::Rabbit);
        assert_eq!(state_of(&check(&a, dt(23, 12)), "transfer_doc"), ItemState::NotApplicable);
        outcome(&mut a, OutcomeMethod::Died);
        assert_eq!(state_of(&check(&a, dt(23, 12)), "transfer_doc"), ItemState::NotApplicable);
        outcome(&mut a, OutcomeMethod::Transferred);
        assert_eq!(state_of(&check(&a, dt(23, 12)), "transfer_doc"), ItemState::Reminder);
        a.disposition.as_mut().unwrap().transfer_doc = Some("BOS-1".into());
        let items = check(&a, dt(23, 12));
        assert_eq!(state_of(&items, "transfer_doc"), ItemState::Done);
        assert_eq!(state_of(&items, "outcome_record"), ItemState::Done);
    }
}
