//! Social posts page: three editable post drafts (one per tone) with Copy buttons,
//! plus the 1080x1080 share image preview and a PNG download.
use crate::app::{href, Route};
use crate::share_image;
use crate::store::AppState;
use leptos::html::{Canvas, Textarea};
use leptos::prelude::*;
use rescue_core::posts::{generate_posts, Post};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// One tone's card. Edits live in the textarea only (demo: nothing is persisted).
#[component]
fn PostCard(post: Post) -> impl IntoView {
    let area = NodeRef::<Textarea>::new();
    let copied = RwSignal::new(false);
    let hint = RwSignal::new(false);
    let label = post.tone.label();

    let copy = move |_| {
        let Some(el) = area.get() else { return };
        let text = el.value();
        // navigator.clipboard is undefined on insecure origins; treat that as a failure.
        let nav = window().navigator();
        let has = js_sys::Reflect::get(&nav, &"clipboard".into()).map(|v| !v.is_undefined() && !v.is_null()).unwrap_or(false);
        // iOS Safari only honours the write if it is issued synchronously inside the click
        // handler, so start it here and only await the resulting promise below.
        let pending = has.then(|| nav.clipboard().write_text(&text));
        leptos::task::spawn_local(async move {
            let ok = match pending {
                Some(p) => JsFuture::from(p).await.is_ok(),
                None => false,
            };
            if ok {
                hint.set(false);
                copied.set(true);
                set_timeout(move || copied.set(false), std::time::Duration::from_secs(2));
            } else {
                // Clipboard blocked: select the text so the user can copy by hand.
                if let Some(el) = area.get() {
                    let _ = el.focus();
                    el.select();
                }
                hint.set(true);
            }
        });
    };

    view! {
        <section class="card post-card">
            <h2>{label}</h2>
            <textarea node_ref=area class="post-text" rows="7" aria-label=format!("{label} post") prop:value=post.text.clone()></textarea>
            <div class="post-actions">
                <button type="button" class="btn primary" on:click=copy>"Copy"</button>
                <span class="post-status" role="status">{move || copied.get().then_some("Copied!")}</span>
            </div>
            {move || hint.get().then(|| view! { <p class="notice">"Press Ctrl+C / long-press to copy"</p> })}
        </section>
    }
}

#[component]
pub fn Posts(id: String) -> impl IntoView {
    let state = expect_context::<AppState>();
    let animal = Memo::new(move |_| state.data.with(|d| d.animals.iter().find(|a| a.id == id).cloned()));
    let canvas = NodeRef::<Canvas>::new();
    let error = RwSignal::new(None::<String>);
    // Mirrors the canvas's `data-ready` attribute so the button can react to it.
    let ready = RwSignal::new(false);
    // The rendered PNG as a data URL, so phones can long-press the <img> to save it.
    let png = RwSignal::new(None::<String>);

    // Re-render the image whenever the animal (or branding) changes; runs after the canvas mounts.
    Effect::new(move |_| {
        let (Some(c), Some(a)) = (canvas.get(), animal.get()) else { return };
        let settings = state.data.with(|d| d.settings.clone());
        error.set(None);
        ready.set(false);
        png.set(None);
        let c2 = c.clone();
        let on_ready = move || {
            png.set(c2.to_data_url_with_type("image/png").ok());
            ready.set(true);
        };
        if let Err(e) = share_image::render_with(&c, &a, &settings, move |e| error.set(Some(e)), on_ready) {
            error.set(Some(format!("Couldn't draw the share image: {e}")));
        }
    });

    let download = move |_| {
        let (Some(c), Some(a)) = (canvas.get(), animal.get_untracked()) else { return };
        let result = (|| -> Result<(), String> {
            let url = c.to_data_url_with_type("image/png").map_err(|e| format!("{e:?}"))?;
            let link: web_sys::HtmlAnchorElement = document().create_element("a").map_err(|e| format!("{e:?}"))?.dyn_into().map_err(|e| format!("{e:?}"))?;
            link.set_href(&url);
            link.set_download(&share_image::download_name(&a.name));
            link.click();
            Ok(())
        })();
        if let Err(e) = result {
            error.set(Some(format!("Couldn't download the image: {e}")));
        }
    };

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
                let posts = generate_posts(&a, &settings);
                let back = format!("\u{2190} Back to {}", a.name);
                let img_alt = format!("Share image for {}", a.name);
                view! {
                    <div class="sheet-actions">
                        <a class="btn" href=href(&Route::Animal(a.id.clone()))>{back}</a>
                    </div>
                    <h1>{format!("Posts for {}", a.name)}</h1>
                    <div class="posts">
                        {posts.into_iter().map(|p| view! { <PostCard post=p /> }).collect_view()}
                    </div>
                    <section class="card share-card">
                        <h2>"Share image"</h2>
                        // The canvas stays in the DOM as the renderer but is hidden once the <img> copy exists.
                        <canvas node_ref=canvas class="share-canvas" aria-label=format!("Share image for {}", a.name)
                            style:display=move || if png.get().is_some() { "none" } else { "block" }></canvas>
                        {move || png.get().map(|url| view! {
                            <img class="share-canvas share-img" src=url alt=img_alt.clone() />
                            <p class="note">"On a phone? Press and hold the image to save it to Photos."</p>
                        })}
                        {move || error.get().map(|e| view! { <p class="notice" role="alert">{e}</p> })}
                        <button type="button" class="btn primary" on:click=download prop:disabled=move || !ready.get()>
                            {move || if ready.get() { "Download image" } else { "Preparing image\u{2026}" }}
                        </button>
                    </section>
                }.into_any()
            }
        }}
    }
}
