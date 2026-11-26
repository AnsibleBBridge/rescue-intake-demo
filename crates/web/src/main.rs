mod app;
mod clock;
mod components;
mod pages;
mod photo;
mod share_image;
mod store;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
