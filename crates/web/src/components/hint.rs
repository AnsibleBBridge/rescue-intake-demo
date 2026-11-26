//! Small "why we ask" disclosure that ties a form field to the Nebraska rule behind it.
use leptos::prelude::*;

#[component]
pub fn Hint(section: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <details class="hint">
            <summary>"Why we ask"</summary>
            <p><strong>{section}</strong>" — "{text}</p>
        </details>
    }
}
