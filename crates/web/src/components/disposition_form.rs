//! Outcome record form: which fields appear depends on how the animal left (23 NAC 18-010.05A
//! for dogs/cats; good practice for other animals).
use crate::clock;
use crate::components::animal_form::{Area, Text};
use crate::components::hint::Hint;
use chrono::NaiveDate;
use leptos::prelude::*;
use rescue_core::model::{Disposition, OutcomeMethod};

const METHODS: [OutcomeMethod; 5] = [
    OutcomeMethod::Adopted,
    OutcomeMethod::Transferred,
    OutcomeMethod::ReturnedToOwner,
    OutcomeMethod::Died,
    OutcomeMethod::Euthanized,
];

fn opt(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

#[component]
pub fn DispositionForm(
    initial: Option<Disposition>,
    on_save: Callback<Disposition>,
    /// Dogs and cats follow 23 NAC 18 (48h); other animals just get a good-practice nudge.
    regulated: bool,
    save_error: RwSignal<Option<String>>,
    /// True while the profile's edit form is open: saving here would discard those edits.
    #[prop(into)] disabled: Signal<bool>,
) -> impl IntoView {
    let d = initial.unwrap_or(Disposition {
        date: Some(clock::now().date()),
        method: OutcomeMethod::Adopted,
        recipient_name: None,
        recipient_address: None,
        circumstances: None,
        euthanasia_by: None,
        euthanasia_method: None,
        transfer_doc: None,
    });
    let sig = |v: &Option<String>| RwSignal::new(v.clone().unwrap_or_default());
    let method = RwSignal::new(d.method);
    let date = RwSignal::new(d.date.map(|x| x.format("%Y-%m-%d").to_string()).unwrap_or_default());
    let (rname, raddr, circ) = (sig(&d.recipient_name), sig(&d.recipient_address), sig(&d.circumstances));
    let (by, how) = (sig(&d.euthanasia_by), sig(&d.euthanasia_method));
    let transfer_doc = sig(&d.transfer_doc);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if disabled.get_untracked() { return; }
        let m = method.get_untracked();
        let recipient = matches!(m, OutcomeMethod::Adopted | OutcomeMethod::Transferred | OutcomeMethod::ReturnedToOwner);
        let euth = m == OutcomeMethod::Euthanized;
        // A bill of sale / adoption contract only exists when the animal went to a new owner.
        let has_doc = matches!(m, OutcomeMethod::Adopted | OutcomeMethod::Transferred);
        // Only keep the fields that apply to the chosen method, so stale text isn't recorded.
        on_save.run(Disposition {
            date: NaiveDate::parse_from_str(&date.get_untracked(), "%Y-%m-%d").ok(),
            method: m,
            recipient_name: if recipient { opt(&rname.get_untracked()) } else { None },
            recipient_address: if recipient { opt(&raddr.get_untracked()) } else { None },
            circumstances: if !recipient { opt(&circ.get_untracked()) } else { None },
            euthanasia_by: if euth { opt(&by.get_untracked()) } else { None },
            euthanasia_method: if euth { opt(&how.get_untracked()) } else { None },
            transfer_doc: if has_doc { opt(&transfer_doc.get_untracked()) } else { None },
        });
    };

    view! {
        <form class="form card" on:submit=on_submit novalidate>
            <h2>"Outcome record"</h2>
            <label class="field"><span class="lbl">"How did the animal leave?"</span>
                <select on:change=move |ev| {
                    let v = event_target_value(&ev);
                    if let Some(m) = METHODS.iter().find(|m| m.label() == v) { method.set(*m); }
                }>
                    {METHODS.iter().map(|m| view! {
                        <option value=m.label() selected=move || method.get() == *m>{m.label()}</option>
                    }).collect_view()}
                </select>
            </label>
            <Text label="Outcome date" value=date kind="date" />
            {move || match method.get() {
                m @ (OutcomeMethod::Adopted | OutcomeMethod::Transferred | OutcomeMethod::ReturnedToOwner) => view! {
                    <Text label="Recipient name" value=rname />
                    <Area label="Recipient mailing address" value=raddr />
                    {matches!(m, OutcomeMethod::Adopted | OutcomeMethod::Transferred).then(|| view! {
                        <Text label="Bill of sale / adoption contract #" value=transfer_doc />
                    })}
                }.into_any(),
                OutcomeMethod::Died => view! {
                    <Area label="Circumstances" value=circ />
                }.into_any(),
                OutcomeMethod::Euthanized => view! {
                    <Area label="Circumstances" value=circ />
                    <Text label="Performed by" value=by />
                    <Text label="Method" value=how />
                }.into_any(),
            }}
            {if regulated {
                view! { <Hint section="23 NAC 18-010.05A" text="Complete within 48 hours of the animal leaving your care." /> }.into_any()
            } else {
                view! { <Hint section="Good practice" text="Write down the date and who the animal went to while you remember it; a signed bill of sale or adoption contract protects you both." /> }.into_any()
            }}
            {move || disabled.get().then(|| view! { <p class="note">"Save or cancel your edits first."</p> })}
            {move || save_error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
            <button type="submit" class="btn primary wide" prop:disabled=move || disabled.get()>"Save Outcome"</button>
        </form>
    }
}
