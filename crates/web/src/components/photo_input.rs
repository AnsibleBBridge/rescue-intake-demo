//! Photo picker with preview. `capture` is deliberately unset so phones offer camera or library.
use crate::photo;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PhotoInput(value: RwSignal<Option<String>>) -> impl IntoView {
    let error = RwSignal::new(None::<String>);
    let on_change = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = event_target(&ev);
        let Some(file) = input.files().and_then(|f| f.get(0)) else { return };
        spawn_local(async move {
            match photo::downscale(file, 1200, 0.8).await {
                Ok(url) => {
                    error.set(None);
                    value.set(Some(url));
                }
                // Keep the previous photo so a bad pick never destroys a good one.
                Err(e) => error.set(Some(e)),
            }
        });
        // Reset so picking the same file again re-fires change.
        input.set_value("");
    };
    view! {
        <div class="field">
            <label class="lbl" for="photo-input">"Photo"</label>
            <input id="photo-input" type="file" accept="image/*" on:change=on_change />
            {move || error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
            {move || value.get().map(|url| view! {
                <div class="preview">
                    <img src=url alt="Photo preview" />
                    <button type="button" class="btn" on:click=move |_| value.set(None)>"Remove photo"</button>
                </div>
            })}
        </div>
    }
}
