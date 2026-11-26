pub mod fact_sheet;
pub mod intake;
pub mod posts;
pub mod profile;
pub mod roster;

use crate::app::Route;
use leptos::prelude::*;

/// Placeholder for routes later tasks will build.
pub fn coming_soon(r: Route) -> impl IntoView {
    view! { <section class="card"><h1>"Coming soon"</h1><p>{format!("{r:?}")}</p></section> }
}
