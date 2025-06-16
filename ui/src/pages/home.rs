use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::component;
use leptos::IntoView;
use leptos::view;

#[component]
pub fn Home() -> impl IntoView {
    view! {
      <h1 class="text-3xl font-bold mb-4 text-neutral-light">"Hello!!!!!!!!!"</h1>

      <div class="h-full flex items-center justify-center bg-neutral-dark text-slate-900">
        <div class="mx-4 p-10 border rounded-2xl shadow-sm bg-neutral-light max-w-lg w-full">
          <div class="flex items-center gap-2 mb-6">
            <img src="/assets/svg/yus.svg" alt="Yus logo" class="h-8"/>
          </div>
          <h1 class="text-4xl font-bold mb-2">Yus Playground</h1>
          <p class="mb-6 text-lg">Choose your path:</p>

          <div class="flex gap-3">
            <a href="/hacker" class="px-6 py-3 rounded-full bg-accent hover:brightness-90 text-text font-medium">Enter Hacker Mode</a>
            <a href="/classic" class="px-6 py-3 rounded-full border border-slate-300 hover:bg-slate-50">Skip to Classic Site</a>
          </div>

          <p class="mt-8 text-sm text-slate-500">Press Y or Enter to activate Hacker Mode.</p>
        </div>
      </div>
    }
}
