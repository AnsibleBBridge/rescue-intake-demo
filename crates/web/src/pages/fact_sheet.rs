//! Printable adoption fact sheet (the rescue's "Pet Profile" layout): title band,
//! Overview card beside a round photo with a name ribbon, Background card, contact band.
use crate::app::{href, Route};
use crate::store::AppState;
use leptos::prelude::*;
use rescue_core::model::*;
use rescue_core::posts::fact_lines;

/// Overview rows in the owner's order; fact_lines already skips unknowns/blanks.
fn overview_rows(a: &Animal) -> Vec<(&'static str, String)> {
    let lines = fact_lines(a);
    ["Breed", "Sex", "Age", "Weight", "Color", "Size potential"]
        .into_iter()
        .filter_map(|label| lines.iter().find(|(k, _)| *k == label).map(|(k, v)| (*k, v.clone())))
        .collect()
}

/// Background card pieces: (personality, "Good with" line, special needs, fee). Missing -> None.
struct Background {
    personality: Option<String>,
    good_with: Option<String>,
    special_needs: Option<String>,
    fee: Option<String>,
}

fn background(a: &Animal) -> Background {
    let s = &a.story;
    let clean = |o: &Option<String>| o.as_ref().map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    let good = fact_lines(a).into_iter().find(|(k, _)| *k == "Good with").map(|(_, v)| format!("Good with: {v}"));
    Background {
        personality: clean(&s.personality),
        good_with: good,
        special_needs: clean(&s.special_needs).map(|t| format!("Special needs: {t}")),
        fee: s.adoption_fee.map(|f| format!("Adoption fee: ${f}")),
    }
}

/// Footer text with blank contact parts dropped; falls back to the rescue's name.
pub fn contact_text(s: &Settings) -> String {
    let parts: Vec<&str> = [s.phone.trim(), s.email.trim()].into_iter().filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        format!("Interested in adopting? Contact {}", s.rescue_name)
    } else {
        let mut it = parts.into_iter();
        let phone_first = !s.phone.trim().is_empty();
        let first = it.next().unwrap_or_default();
        let head = if phone_first { format!("Interested in adopting? Call {first}") } else { format!("Interested in adopting? Email {first}") };
        match it.next() {
            Some(second) => format!("{head} \u{b7} {second}"),
            None => head,
        }
    }
}

/// Silhouette outline (100x100 viewBox), shared with the share-image canvas.
pub fn sil_path(species: Species) -> &'static str {
    match species {
        Species::Cat => "M22 14 L40 32 Q50 29 60 32 L78 14 L80 48 Q86 60 78 74 Q66 90 50 90 Q34 90 22 74 Q14 60 20 48 Z",
        Species::Dog => "M26 24 Q38 22 42 34 Q50 32 58 34 Q62 22 74 24 Q88 30 82 56 Q76 52 74 46 Q76 70 62 82 Q50 92 38 82 Q24 70 26 46 Q24 52 18 56 Q12 30 26 24 Z",
    }
}

/// Simple silhouette used when there is no photo (kept inline so print needs no assets).
fn silhouette(species: Species) -> impl IntoView {
    match species {
        Species::Cat => view! {
            <svg class="sil" viewBox="0 0 100 100" aria-hidden="true">
                <path d="M22 14 L40 32 Q50 29 60 32 L78 14 L80 48 Q86 60 78 74 Q66 90 50 90 Q34 90 22 74 Q14 60 20 48 Z" fill="currentColor"/>
            </svg>
        }.into_any(),
        Species::Dog => view! {
            <svg class="sil" viewBox="0 0 100 100" aria-hidden="true">
                <path d="M26 24 Q38 22 42 34 Q50 32 58 34 Q62 22 74 24 Q88 30 82 56 Q76 52 74 46 Q76 70 62 82 Q50 92 38 82 Q24 70 26 46 Q24 52 18 56 Q12 30 26 24 Z" fill="currentColor"/>
            </svg>
        }.into_any(),
    }
}

#[component]
pub fn FactSheet(id: String) -> impl IntoView {
    let state = expect_context::<AppState>();
    let animal = Memo::new(move |_| state.data.with(|d| d.animals.iter().find(|a| a.id == id).cloned()));

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
                let settings = state.data.with(|d| d.settings.clone());
                let rows = overview_rows(&a);
                let bg = background(&a);
                let bg_empty = bg.personality.is_none() && bg.good_with.is_none() && bg.special_needs.is_none() && bg.fee.is_none();
                let photo = match a.photo.clone() {
                    Some(src) => view! { <img src=src alt=format!("Photo of {}", a.name) /> }.into_any(),
                    None => view! { <div class="sheet-ph">{silhouette(a.species)}</div> }.into_any(),
                };
                let back = format!("\u{2190} Back to {}", a.name);
                view! {
                    <div class="sheet-actions">
                        <a class="btn" href=href(&Route::Animal(a.id.clone()))>{back}</a>
                        <button type="button" class="btn primary" on:click=|_| { let _ = window().print(); }>"Print"</button>
                    </div>
                    <div class="sheet-wrap"><article class="sheet">
                        <header class="sheet-title">
                            <h1>"Pet Profile"</h1>
                            <p>{settings.rescue_name.clone()}</p>
                        </header>
                        <div class="sheet-top">
                            <section class="sheet-card">
                                <h2>"Overview"</h2>
                                <dl class="sheet-dl">
                                    {rows.into_iter().map(|(k, v)| view! { <dt>{k}</dt><dd>{v}</dd> }).collect_view()}
                                </dl>
                            </section>
                            <div class="sheet-photo">
                                <div class="circle">{photo}</div>
                                <div class="ribbon">{a.name.clone()}</div>
                            </div>
                        </div>
                        <section class="sheet-card sheet-bg">
                            <h2>"Background"</h2>
                            {bg.personality.map(|t| view! { <p>{t}</p> })}
                            {bg.good_with.map(|t| view! { <p>{t}</p> })}
                            {bg.special_needs.map(|t| view! { <p>{t}</p> })}
                            {bg.fee.map(|t| view! { <p class="fee">{t}</p> })}
                            {bg_empty.then(|| view! { <p>{format!("Ask us about {}!", a.name)}</p> })}
                        </section>
                        <footer class="sheet-foot">{contact_text(&settings)}</footer>
                    </article></div>
                }.into_any()
            }
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_drops_blank_parts() {
        let mut s = Settings { rescue_name: "Paws".into(), phone: "".into(), email: "".into(), ..Default::default() };
        assert_eq!(contact_text(&s), "Interested in adopting? Contact Paws");
        s.phone = "555-1234".into();
        assert_eq!(contact_text(&s), "Interested in adopting? Call 555-1234");
        s.email = "a@b.org".into();
        assert_eq!(contact_text(&s), "Interested in adopting? Call 555-1234 \u{b7} a@b.org");
        s.phone = " ".into();
        assert_eq!(contact_text(&s), "Interested in adopting? Email a@b.org");
    }
}
