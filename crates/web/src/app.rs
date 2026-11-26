//! Hash router + page layout (banner, header, footer).
use crate::pages;
use crate::store::{AppState, LocalStore};
use leptos::prelude::*;
use rescue_core::model::AnimalId;

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Roster,
    New,
    Animal(AnimalId),
    Sheet(AnimalId),
    Posts(AnimalId),
    Settings,
    NotFound,
}

/// Pure hash -> Route mapping (host-testable; no browser needed).
pub fn parse_hash(h: &str) -> Route {
    let path = h.strip_prefix('#').unwrap_or(h);
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match parts.as_slice() {
        [] => Route::Roster,
        ["new"] => Route::New,
        ["settings"] => Route::Settings,
        ["animal", id] => Route::Animal((*id).to_string()),
        ["animal", id, "sheet"] => Route::Sheet((*id).to_string()),
        ["animal", id, "posts"] => Route::Posts((*id).to_string()),
        _ => Route::NotFound,
    }
}

/// Route -> hash string; inverse of `parse_hash` for every real route.
pub fn href(r: &Route) -> String {
    match r {
        Route::Roster | Route::NotFound => "#/".into(),
        Route::New => "#/new".into(),
        Route::Animal(id) => format!("#/animal/{id}"),
        Route::Sheet(id) => format!("#/animal/{id}/sheet"),
        Route::Posts(id) => format!("#/animal/{id}/posts"),
        Route::Settings => "#/settings".into(),
    }
}

fn current_hash() -> String {
    window().location().hash().unwrap_or_default()
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(AppState::init(Box::new(LocalStore)));

    // Re-parse on every hashchange so back/forward and typed URLs work.
    let route = RwSignal::new(parse_hash(&current_hash()));
    window_event_listener(leptos::ev::hashchange, move |_| {
        route.set(parse_hash(&current_hash()));
    });

    view! {
        <div class="banner" role="note">"Demo — entries stay in this browser only."</div>
        <header class="topbar">
            <a class="brand" href=href(&Route::Roster)>"Rescue Intake"</a>
            <nav>
                <a href=href(&Route::Roster)>"Animals"</a>
                <a href=href(&Route::Settings)>"Settings"</a>
            </nav>
        </header>
        <main class="page">
            {move || match route.get() {
                Route::Roster => pages::roster::Roster().into_any(),
                Route::New => pages::intake::Intake().into_any(),
                Route::Animal(id) => pages::profile::Profile(pages::profile::ProfileProps { id }).into_any(),
                Route::Sheet(id) => pages::fact_sheet::FactSheet(pages::fact_sheet::FactSheetProps { id }).into_any(),
                Route::Posts(id) => pages::posts::Posts(pages::posts::PostsProps { id }).into_any(),
                Route::Settings => pages::settings::Settings().into_any(),
                Route::NotFound => pages::not_found().into_any(),
            }}
        </main>
        <footer class="footer">
            "Demo only — not legal advice. Confirm current requirements with the Nebraska Department of Agriculture."
        </footer>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_posts_route() {
        assert_eq!(parse_hash("#/animal/abc/posts"), Route::Posts("abc".into()));
    }

    #[test]
    fn empty_and_root_are_roster() {
        assert_eq!(parse_hash(""), Route::Roster);
        assert_eq!(parse_hash("#/"), Route::Roster);
    }

    #[test]
    fn unknown_is_not_found() {
        assert_eq!(parse_hash("#/bogus"), Route::NotFound);
        assert_eq!(parse_hash("#/animal"), Route::NotFound);
    }

    #[test]
    fn href_round_trips() {
        for h in ["#/", "#/new", "#/animal/x1", "#/animal/x1/sheet", "#/animal/x1/posts", "#/settings"] {
            assert_eq!(href(&parse_hash(h)), h);
        }
    }
}
