//! Animal profile: header, status stepper, compliance card, outcome form, details/edit.
use crate::app::{href, Route};
use crate::clock;
use crate::components::{
    animal_form::AnimalForm, compliance_list::ComplianceList, disposition_form::DispositionForm,
    status_stepper::StatusStepper,
};
use crate::store::AppState;
use leptos::prelude::*;
use rescue_core::compliance;
use rescue_core::model::*;
use rescue_core::posts::fact_lines;

/// (label, value) pairs worth showing; blanks are dropped so the summary stays short.
fn detail_rows(a: &Animal) -> Vec<(&'static str, String)> {
    let mut rows: Vec<(&'static str, Option<String>)> = Vec::new();
    rows.push(("Received", Some(a.received_at.format("%b %-d, %Y %-I:%M %p").to_string())));
    rows.push((
        "Source",
        a.source.as_ref().map(|s| match s {
            Source::Person { name, address, drivers_license } => {
                if drivers_license.trim().is_empty() {
                    format!("{name}, {address} (no driver's license yet)")
                } else {
                    format!("{name}, {address} (DL {drivers_license})")
                }
            }
            Source::Agency { agency, official_title, address } => format!("{agency}, {official_title}, {address}"),
        }),
    ));
    rows.push(("Certificate of veterinary inspection", a.health_certificate.clone()));
    rows.push(("Proof of ownership", a.ownership_doc.clone()));
    rows.push(("Scrapie tag", a.scrapie_tag.clone()));
    rows.push((
        "Coggins",
        a.coggins.as_ref().and_then(|c| {
            let parts: Vec<String> = [c.date.map(|d| d.format("%b %-d, %Y").to_string()), c.lab.clone(), c.result.clone()]
                .into_iter().flatten().filter(|p| !p.trim().is_empty()).collect();
            (!parts.is_empty()).then(|| parts.join(" · "))
        }),
    ));
    let id = &a.identification;
    rows.push(("Tag", id.tag.clone()));
    rows.push(("Cage card", id.cage_card.clone()));
    rows.push(("Tattoo", id.tattoo.clone()));
    rows.push((
        "Microchip",
        id.microchip.as_ref().map(|m| {
            let extra: Vec<&str> = [m.manufacturer.as_deref(), m.location.as_deref()].into_iter().flatten().collect();
            if extra.is_empty() { m.number.clone() } else { format!("{} ({})", m.number, extra.join(", ")) }
        }),
    ));
    let s = &a.story;
    // Same wording as the fact sheet and posts; the fee has its own row below.
    let facts = fact_lines(a);
    for (k, v) in facts.iter().filter(|(k, _)| *k != "Fee") {
        rows.push((k, Some(v.clone())));
    }
    rows.push(("Birth date", s.birth_date.map(|d| d.format("%b %-d, %Y").to_string())));
    rows.push(("Personality", s.personality.clone()));
    rows.push(("Special needs", s.special_needs.clone()));
    rows.push(("Adoption fee", s.adoption_fee.map(|f| format!("${f}"))));
    rows.into_iter().filter_map(|(k, v)| v.filter(|v| !v.trim().is_empty()).map(|v| (k, v))).collect()
}

fn health_line(h: &HealthEntry) -> String {
    let mut t = format!("{} — {}", h.date.format("%b %-d, %Y"), h.description);
    for extra in [&h.reason, &h.result].into_iter().flatten() {
        t.push_str(&format!(" · {extra}"));
    }
    t
}

#[component]
pub fn Profile(id: String) -> impl IntoView {
    let state = expect_context::<AppState>();
    let editing = RwSignal::new(false);
    // One banner for any failed save on this page (status, outcome, or edit).
    let save_error = RwSignal::new(None::<String>);
    let animal = Memo::new({
        let id = id.clone();
        move |_| state.data.with(|d| d.animals.iter().find(|a| a.id == id).cloned())
    });

    view! {
        {move || match animal.get() {
            None => view! {
                <section class="card">
                    <h1>"We couldn't find that animal"</h1>
                    <p>"It may have been removed, or the link is mistyped."</p>
                    <a class="btn primary" href=href(&Route::Roster)>"Back to the roster"</a>
                </section>
            }.into_any(),
            Some(a) => {
                let now = clock::now();
                // Every mutation goes through here so failures always surface on the page.
                let save = move |next: Animal| -> bool {
                    save_error.set(None);
                    match state.upsert(next) {
                        Ok(()) => true,
                        Err(e) => { save_error.set(Some(e)); false }
                    }
                };
                let current = a.clone();
                let on_pick = Callback::new(move |s: Status| {
                    // Saving the stale snapshot would discard in-progress edits.
                    if s == current.status || editing.get_untracked() { return; }
                    let mut n = current.clone();
                    n.status = s;
                    n.status_changed_at = clock::now();
                    save(n);
                });
                let on_outcome = {
                    let base = a.clone();
                    Callback::new(move |d: Disposition| {
                        let mut n = base.clone();
                        n.disposition = Some(d);
                        save(n);
                    })
                };
                let on_edit_save = Callback::new(move |n: Animal| {
                    if save(n) { editing.set(false); }
                });

                let photo = match a.photo.clone() {
                    Some(src) => view! { <img src=src alt=format!("Photo of {}", a.name) /> }.into_any(),
                    None => {
                        let c = crate::placeholder::emoji(a.species.family());
                        view! { <div class="ph" aria-hidden="true">{c}</div> }.into_any()
                    }
                };
                let breed = a.story.breed.clone().unwrap_or_else(|| "Unknown breed".into());
                let items = compliance::check(&a, now);
                let rows = detail_rows(&a);
                let regulated = matches!(a.species.family(), Family::Dog | Family::Cat);
                // Only dogs and cats have 48-hour deadlines; say so before the softer reminders.
                let farm_note = (!regulated).then(|| view! { <p class="sub">{crate::components::animal_form::FARM_NOTE}</p> });
                let health: Vec<String> = a.health.iter().map(health_line).collect();
                let edit_lock = Signal::derive(move || editing.get());
                let status_sig = Signal::derive({ let s = a.status; move || s });
                let edit_src = a.clone();
                let outcome_form = (a.status == Status::Outcome).then(|| view! {
                    <DispositionForm regulated=regulated initial=a.disposition.clone() on_save=on_outcome save_error=save_error disabled=edit_lock />
                });

                view! {
                    <section class="profile-head">
                        <div class="photo hero">{photo}</div>
                        <h1>{a.name.clone()}</h1>
                        <p class="sub">{format!("{} · {breed}", a.kind_label())}</p>
                        <span class=crate::pages::roster::status_class(a.status)>{crate::pages::roster::status_text(&a)}</span>
                    </section>
                    {move || save_error.get().filter(|_| !editing.get()).map(|e| view! { <p class="notice" role="alert">{e}</p> })}
                    <section class="card">
                        <h2>"Status"</h2>
                        <StatusStepper current=status_sig on_pick=on_pick disabled=edit_lock />
                        {move || editing.get().then(|| view! { <p class="note" data-testid="edit-lock">"Save or cancel your edits first."</p> })}
                    </section>
                    {outcome_form}
                    <section class="card">
                        <h2>"Records & reminders"</h2>
                        {farm_note}
                        <ComplianceList items=items now=now />
                    </section>
                    <div class="chips">
                        <a class="btn" href=href(&Route::Sheet(a.id.clone()))>"Fact sheet"</a>
                        <a class="btn" href=href(&Route::Posts(a.id.clone()))>"Social posts"</a>
                    </div>
                    <section class="card">
                        <div class="row">
                            <h2>"Details"</h2>
                            {move || (!editing.get()).then(|| view! {
                                <button type="button" class="btn" on:click=move |_| { save_error.set(None); editing.set(true); }>"Edit"</button>
                            })}
                        </div>
                        {move || if editing.get() {
                            view! {
                                <AnimalForm initial=edit_src.clone() on_save=on_edit_save submit_label="Save changes" save_error=save_error />
                                <button type="button" class="btn wide" on:click=move |_| { save_error.set(None); editing.set(false); }>"Cancel"</button>
                            }.into_any()
                        } else {
                            view! {
                                <dl class="details">
                                    {rows.clone().into_iter().map(|(k, v)| view! { <dt>{k}</dt><dd>{v}</dd> }).collect_view()}
                                </dl>
                                {(!health.is_empty()).then(|| view! {
                                    <h3>"Health record"</h3>
                                    <ul class="plain">{health.iter().map(|h| view! { <li>{h.clone()}</li> }).collect_view()}</ul>
                                })}
                            }.into_any()
                        }}
                    </section>
                }.into_any()
            }
        }}
    }
}
