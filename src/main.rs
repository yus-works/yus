use leptos::*;
use leptos::prelude::ElementChild;
use mount::mount_to_body;

fn main() {
    mount_to_body(|| {
        view! {
            <h1>"Hello from Leptos!"</h1>
        }
    });
}
