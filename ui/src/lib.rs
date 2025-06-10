use leptos::*;
use mount::mount_to_body;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

pub mod app;
pub mod routes;
pub mod pages;
pub mod render;
pub mod components;

pub use crate::app::App;
