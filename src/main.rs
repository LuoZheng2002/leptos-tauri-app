use crate::app::App;
use leptos::prelude::*;

pub mod app;
pub mod components;
pub mod models;
pub mod pages;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App /> }
    })
}
