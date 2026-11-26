mod app;
mod clock;
mod pages;
mod store;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
