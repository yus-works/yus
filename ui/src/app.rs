use leptos::*;
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use crate::routes::RoutesMenu;
use leptos_router::components::Router;
use leptos_router::components::A;

use leptos_meta::provide_meta_context;
use leptos_meta::Stylesheet;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();      // NOTE: sets up <head> manager ?

    view! {
      <Stylesheet href="/pkg/ui.css"/>
      <Router>
        <header class="p-4 flex gap-4 bg-neutral-950 text-neutral-50">
          <A href="/">"Yus"</A>
          <A href="/demos">"Demos"</A>
        </header>

        <main class="min-h-screen p-4">
          <RoutesMenu/>
        </main>

        <footer class="p-4 text-center bg-neutral-950 text-neutral-50">
          "© 2025 Yus – curiosity compiled"
        </footer>
      </Router>
    }
}
