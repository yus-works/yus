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
      <Router>
        <header class="sticky top-0 bg-surface/85 backdrop-blur">
          <div class="max-w-6xl mx-auto flex justify-between items-center px-6 py-4">
            <a href="/" class="text-2xl font-extrabold text-primary">YUS</a>
            <nav class="hidden md:flex gap-8 text-text">
              <a href="/projects">Projects</a>
              <a href="/about">About</a>
              <a href="/contact">Contact</a>
            </nav>
          </div>
        </header>

        <main class="min-h-screen p-4">
          <RoutesMenu/>
        </main>

        <footer class="bg-surface text-text py-8">
          <div class="max-w-6xl mx-auto px-6 flex flex-col sm:flex-row justify-between gap-8">
            <p>"© 2025 Yus — Ideas in orbit; thoughts worth circling back to."</p>
            <nav class="flex gap-6 underline-offset-4">
              <a href="https://github.com/yus-works" target="_blank">GitHub</a>
              <a href="/contact">Contact</a>
            </nav>
          </div>
        </footer>
      </Router>
    }
}
