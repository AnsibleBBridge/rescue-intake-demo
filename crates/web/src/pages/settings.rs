//! Rescue settings: the branding shown on fact sheets, social posts and share images.
use crate::store::AppState;
use leptos::prelude::*;
use rescue_core::model::Settings as RescueSettings;

/// "adopt  #nebraska" -> ["#adopt", "#nebraska"]: split on whitespace, force a leading '#',
/// drop empties (a lone "#" counts as empty).
pub fn parse_hashtags(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|t| t.trim_start_matches('#'))
        .filter(|t| !t.is_empty())
        .map(|t| format!("#{t}"))
        .collect()
}

#[component]
fn Field(label: &'static str, value: RwSignal<String>, saved: RwSignal<bool>) -> impl IntoView {
    view! {
        <label class="field"><span class="lbl">{label}</span>
            <input type="text" prop:value=move || value.get()
                on:input=move |ev| { value.set(event_target_value(&ev)); saved.set(false); } />
        </label>
    }
}

#[component]
pub fn Settings() -> impl IntoView {
    let state = expect_context::<AppState>();
    let cur = state.data.with_untracked(|d| d.settings.clone());
    let name = RwSignal::new(cur.rescue_name);
    let phone = RwSignal::new(cur.phone);
    let email = RwSignal::new(cur.email);
    let website = RwSignal::new(cur.website);
    let tags = RwSignal::new(cur.hashtags.join(" "));
    let saved = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let s = RescueSettings {
            rescue_name: name.get_untracked().trim().to_string(),
            phone: phone.get_untracked().trim().to_string(),
            email: email.get_untracked().trim().to_string(),
            website: website.get_untracked().trim().to_string(),
            hashtags: parse_hashtags(&tags.get_untracked()),
        };
        match state.save_settings(s) {
            Ok(()) => {
                error.set(None);
                saved.set(true);
            }
            Err(e) => {
                saved.set(false);
                error.set(Some(e));
            }
        }
    };

    view! {
        <h1>"Settings"</h1>
        <form class="form card settings" on:submit=on_submit>
            <p class="note">"Shown on fact sheets, social posts and share images. Stored only in this browser."</p>
            <Field label="Rescue name" value=name saved=saved />
            <Field label="Phone" value=phone saved=saved />
            <Field label="Email" value=email saved=saved />
            <Field label="Website" value=website saved=saved />
            <Field label="Hashtags" value=tags saved=saved />
            <p class="hint">"Separate hashtags with spaces; the # is added for you."</p>
            {move || error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
            <div class="post-actions">
                <button type="submit" class="btn primary">"Save"</button>
                <span class="post-status" role="status">{move || saved.get().then_some("Saved")}</span>
            </div>
        </form>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_missing_hash_and_splits_on_whitespace() {
        assert_eq!(parse_hashtags("adopt  #nebraska\tfoster"), vec!["#adopt", "#nebraska", "#foster"]);
    }

    #[test]
    fn drops_empties_and_bare_hashes() {
        assert!(parse_hashtags("   ").is_empty());
        assert_eq!(parse_hashtags("# ## #ok"), vec!["#ok"]);
    }
}
