//! Status pills in pipeline order; the current one is highlighted, any pill is clickable.
use leptos::prelude::*;
use rescue_core::model::Status;

#[component]
pub fn StatusStepper(current: Signal<Status>, on_pick: Callback<Status>) -> impl IntoView {
    view! {
        <div class="chips stepper" role="group" aria-label="Status">
            {Status::ALL.map(|s| view! {
                <button type="button" class="chip" class:active=move || current.get() == s
                    aria-pressed=move || (current.get() == s).to_string()
                    on:click=move |_| on_pick.run(s)>{s.label()}</button>
            }).collect_view()}
        </div>
    }
}
