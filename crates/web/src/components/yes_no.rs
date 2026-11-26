//! Three-way Yes / No / Not sure toggle for facts a rescue often can't know.
use leptos::prelude::*;
use rescue_core::model::YesNoUnknown;

#[component]
pub fn YesNo(value: RwSignal<YesNoUnknown>, label: &'static str) -> impl IntoView {
    let opts = [
        (YesNoUnknown::Yes, "Yes"),
        (YesNoUnknown::No, "No"),
        (YesNoUnknown::Unknown, "Not sure"),
    ];
    view! {
        <div class="field" role="group" aria-label=label>
            <span class="lbl">{label}</span>
            <div class="chips tight">
                {opts.into_iter().map(|(v, text)| view! {
                    <button type="button" class="chip" class:active=move || value.get() == v
                        aria-pressed=move || (value.get() == v).to_string()
                        on:click=move |_| value.set(v)>{text}</button>
                }).collect_view()}
            </div>
        </div>
    }
}
