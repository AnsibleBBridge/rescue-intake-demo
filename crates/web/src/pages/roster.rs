//! Roster: card grid of animals with status filter chips.
use crate::app::{href, Route};
use crate::store::AppState;
use leptos::prelude::*;
use rescue_core::model::{Animal, Status};

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    InCare,
    Available,
    Outcomes,
    All,
}

impl Filter {
    const ALL: [Filter; 4] = [Filter::InCare, Filter::Available, Filter::Outcomes, Filter::All];

    fn label(&self) -> &'static str {
        match self {
            Filter::InCare => "In care",
            Filter::Available => "Available",
            Filter::Outcomes => "Outcomes",
            Filter::All => "All",
        }
    }

    fn matches(&self, s: Status) -> bool {
        match self {
            Filter::InCare => s != Status::Outcome,
            Filter::Available => s == Status::Available,
            Filter::Outcomes => s == Status::Outcome,
            Filter::All => true,
        }
    }
}

fn status_class(s: Status) -> &'static str {
    match s {
        Status::Intake => "badge b-intake",
        Status::VetCheck => "badge b-vet",
        Status::MedicalHold => "badge b-hold",
        Status::Available => "badge b-avail",
        Status::AdoptionPending => "badge b-pending",
        Status::Outcome => "badge b-outcome",
    }
}

#[component]
pub fn Roster() -> impl IntoView {
    let state = expect_context::<AppState>();
    let filter = RwSignal::new(Filter::All);

    let on_reset = move |_| {
        let ok = window()
            .confirm_with_message("Reset demo? This replaces all entries with the 3 sample animals.")
            .unwrap_or(false);
        if ok {
            state.reset();
        }
    };

    view! {
        <div class="row">
            <h1>"Animals"</h1>
            <a class="btn primary" href=href(&Route::New)>"+ New intake"</a>
        </div>
        {move || state.notice.get().map(|n| view! { <p class="notice" role="status">{n}</p> })}
        <div class="chips" role="group" aria-label="Filter animals">
            {Filter::ALL.map(|f| view! {
                <button class="chip" type="button"
                    class:active=move || filter.get() == f
                    aria-pressed=move || (filter.get() == f).to_string()
                    on:click=move |_| filter.set(f)>{f.label()}</button>
            }).collect_view()}
        </div>
        <div class="grid">
            {move || {
                let f = filter.get();
                let animals: Vec<Animal> = state.data.with(|d| {
                    d.animals.iter().filter(|a| f.matches(a.status)).cloned().collect()
                });
                if animals.is_empty() {
                    view! { <p class="empty">"No animals here yet."</p> }.into_any()
                } else {
                    animals.into_iter().map(card).collect_view().into_any()
                }
            }}
        </div>
        <p class="actions"><button class="btn" type="button" on:click=on_reset>"Reset demo"</button></p>
    }
}

fn card(a: Animal) -> impl IntoView {
    let link = href(&Route::Animal(a.id.clone()));
    let breed = a.story.breed.clone().unwrap_or_else(|| "Unknown breed".into());
    let species = format!("{:?}", a.species);
    let initial = a.name.chars().next().unwrap_or('?').to_string();
    let photo = match a.photo.clone() {
        Some(src) => view! { <img src=src alt=format!("Photo of {}", a.name) /> }.into_any(),
        None => view! { <div class="ph" aria-hidden="true">{initial}</div> }.into_any(),
    };
    view! {
        <a class="card animal-card" href=link>
            <div class="photo">{photo}</div>
            <h2>{a.name.clone()}</h2>
            <p class="sub">{format!("{species} · {breed}")}</p>
            <span class=status_class(a.status)>{a.status.label()}</span>
        </a>
    }
}
