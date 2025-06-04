use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::view;
use leptos::{IntoView, component};

#[component]
pub fn classic_main() -> impl IntoView {
    view! {

      <main class="max-w-6xl mx-auto px-6">
        // Hero
        <section class="py-24">
          <h1 class="text-5xl sm:text-7xl font-display text-text mb-6 leading-tight">
            I build things<br class="hidden sm:block"/> you can poke.
          </h1>
          <p class="font-sans text-xl mb-10 max-w-xl text-text">
          "
            Rockets in Minecraft, motion rigs for wannabe rally drivers,
            and (coming soon) a real-life CS:GO airsoft map. Pick one ↓
          "
          </p>
          <a href="#projects"
             class="inline-block bg-primary text-text px-8 py-4 rounded-full
                    hover:bg-primary transition">View Projects</a>
        </section>

        // Project cards
        <section id="projects" class="py-16 grid sm:grid-cols-3 gap-8">
          // repeat card -->
          <article class="bg-white rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Minecraft Rocket Mod</h3>
              <p class="text-sm text-slate-700">GPL-3.0, Kerbal-style physics.</p>
            </div>
          </article>
          <article class="bg-white rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Minecraft Rocket Mod</h3>
              <p class="text-sm text-slate-700">GPL-3.0, Kerbal-style physics.</p>
            </div>
          </article>
          <article class="bg-white rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Minecraft Rocket Mod</h3>
              <p class="text-sm text-slate-700">GPL-3.0, Kerbal-style physics.</p>
            </div>
          </article>
          // ... -->
        </section>

        <section class="py-20">
          <h2 class="text-3xl font-bold mb-8">WebGPU demo</h2>
          <div class="grid md:grid-cols-2 gap-6">
            <textarea id="shader" class="w-full h-72 bg-surface text-text p-4 font-mono
                                         rounded-xl resize-none"></textarea>
            <canvas id="wgpu" class="w-full h-72 rounded-xl border"></canvas>
          </div>
        </section>
      </main>

      <script type="module" src="/js/demo.js"></script>
    }
}
