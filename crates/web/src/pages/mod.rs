pub mod fact_sheet;
pub mod intake;
pub mod posts;
pub mod profile;
pub mod roster;
pub mod settings;

use crate::app::{href, Route};
use leptos::prelude::*;

/// Shown for any hash that matches no route.
pub fn not_found() -> impl IntoView {
    view! {
        <section class="card">
            <h1>"Page not found"</h1>
            <p>"That address doesn't match anything here."</p>
            <a class="btn primary" href=href(&Route::Roster)>"Back to the roster"</a>
        </section>
    }
}
