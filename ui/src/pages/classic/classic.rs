use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::view;
use leptos::{IntoView, component};

#[component]
pub fn classic_main() -> impl IntoView {
    view! {
      <header class="sticky top-0 bg-neutral-light/80 backdrop-blur">
        <div class="max-w-6xl mx-auto flex justify-between items-center px-6 py-4">
          <a href="/" class="text-2xl font-extrabold text-primary">YUS</a>
          <nav class="hidden md:flex gap-8 text-neutral-dark">
            <a href="#projects">Projects</a>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
          </nav>
        </div>
      </header>

      <main class="max-w-6xl mx-auto px-6">
        // Hero
        <section class="py-24">
          <h1 class="text-5xl sm:text-7xl font-bold text-neutral-light mb-6 leading-tight">
            I build things<br class="hidden sm:block"/> you can poke.
          </h1>
          <p class="text-xl mb-10 max-w-xl text-neutral-light">
          "
            Rockets in Minecraft, motion rigs for wannabe rally drivers,
            and (coming soon) a real-life CS:GO airsoft map. Pick one ↓
          "
          </p>
          <a href="#projects"
             class="inline-block bg-primary text-white px-8 py-4 rounded-full
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
            <textarea id="shader" class="w-full h-72 bg-neutral-dark text-neutral-light p-4 font-mono
                                         rounded-xl resize-none"></textarea>
            <canvas id="wgpu" class="w-full h-72 rounded-xl border"></canvas>
          </div>
        </section>
      </main>

      <footer class="bg-neutral-dark text-neutral-light py-12">
        <div class="max-w-6xl mx-auto px-6 flex flex-col sm:flex-row justify-between gap-8">
          <p>"© 2025 Teodor Đurić - idk yet."</p>
          <nav class="flex gap-6 underline-offset-4">
            <a href="https://github.com/…" target="_blank">GitHub</a>
            <a href="/contact">Contact</a>
          </nav>
        </div>
      </footer>
      <script type="module" src="/js/demo.js"></script>
    }
}
