mod app;
mod clock;
mod components;
mod pages;
mod photo;
mod store;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
