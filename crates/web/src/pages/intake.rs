//! New-animal intake: a blank animal in the shared form; saving goes to the profile.
use crate::app::{href, Route};
use crate::clock;
use crate::components::animal_form::AnimalForm;
use crate::store::AppState;
use chrono::NaiveDateTime;
use leptos::prelude::*;
use rescue_core::model::*;

/// The three fields Nebraska's 48-hour window lets us require up front.
/// Err lists the labels of whatever is missing.
pub fn validate_minimum(
    name: &str,
    species: Option<Species>,
    received: Option<NaiveDateTime>,
) -> Result<(), Vec<&'static str>> {
    let mut missing = Vec::new();
    if name.trim().is_empty() {
        missing.push("Name");
    }
    if species.is_none() {
        missing.push("Species");
    }
    if received.is_none() {
        missing.push("Date received");
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

#[component]
pub fn Intake() -> impl IntoView {
    let state = expect_context::<AppState>();
    let save_error = RwSignal::new(None::<String>);
    let now = clock::now();
    let blank = Animal {
        id: uuid::Uuid::new_v4().to_string(),
        name: String::new(),
        species: Species::Dog,
        species_other: None,
        sex: Sex::Unknown,
        received_at: now,
        source: None,
        out_of_state: false,
        health_certificate: None,
        ownership_doc: None,
        coggins: None,
        scrapie_tag: None,
        identification: Identification::default(),
        health: Vec::new(),
        story: Story::default(),
        photo: None,
        status: Status::Intake,
        status_changed_at: now,
        disposition: None,
    };
    let on_save = Callback::new(move |mut a: Animal| {
        // The form may have edited the received time; keep the status clock in step.
        a.status_changed_at = a.received_at;
        let id = a.id.clone();
        match state.upsert(a) {
            Ok(()) => {
                let _ = window().location().set_hash(&href(&Route::Animal(id)));
            }
            Err(e) => save_error.set(Some(e)),
        }
    });
    view! {
        <h1>"New intake"</h1>
        <AnimalForm initial=blank on_save=on_save submit_label="Save intake" save_error=save_error />
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn when() -> Option<NaiveDateTime> {
        NaiveDate::from_ymd_opt(2026, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5))
    }

    #[test]
    fn validate_minimum_requires_three_fields() {
        assert_eq!(validate_minimum("", Some(Species::Dog), when()), Err(vec!["Name"]));
        assert_eq!(validate_minimum("   ", None, None), Err(vec!["Name", "Species", "Date received"]));
        assert_eq!(validate_minimum("Rex", Some(Species::Cat), when()), Ok(()));
    }
}
