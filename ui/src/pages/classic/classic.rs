use leptos::prelude::ElementChild;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::ClassAttribute;
use leptos::prelude::view;
use leptos::{IntoView, component};

use crate::pages::demos::cube::CubeDemo;

#[component]
pub fn classic_main() -> impl IntoView {
    view! {
      <main class="max-w-6xl mx-auto px-6">
        <section class="py-24">
          <h1 class="text-5xl sm:text-7xl font-display text-text mb-6 leading-tight">
            Curious by nature,<br/>Serious by habit.
          </h1>
          <a href="#projects"
             class="inline-block bg-primary text-neutral-dark px-8 py-4 rounded-full
                    hover:bg-primary transition">View Projects</a>
        </section>

        // Project cards
        <section id="projects" class="py-16 grid sm:grid-cols-3 gap-8">
          // repeat card -->
          <article class="bg-neutral-light rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Velari</h3>
              <p class="text-sm text-slate-700">Minecraft Space Travel Mod<br/>GPL-3.0</p>
            </div>
          </article>
          <article class="bg-neutral-light rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Metal Stars</h3>
              <p class="text-sm text-slate-700">AR Satellite Visualizer<br/>GPL-3.0</p>
            </div>
          </article>
          <article class="bg-neutral-light rounded-xl overflow-hidden shadow">
            <img src="/img/rocket.jpg" alt="" class="h-40 w-full object-cover"/>
            <div class="p-4">
              <h3 class="font-semibold text-lg mb-1">Yus Experiments</h3>
              <p class="text-sm text-slate-700">Experiments that test my abilities</p>
            </div>
          </article>
        </section>

        <section id="experiments" class="py-8">
          <h2 class="text-3xl font-bold text-text">Yus Experiments</h2>

          <p class="text-text leading-relaxed">
            "Needed a cube-planet visualiser for my Minecraft space-mod "
            <a href="/velari" class="underline hover:text-[#E55934]">Velari</a>". "
            "I picked up "<strong>wgpu</strong>", liked it too much, and ported the toy to WebGPU."<br/>
            <strong>"Now poke the prototype below 👇"</strong>
          </p>
        </section>

        <section id="shader-lab" class="py-8">
          <h2 class="text-3xl text-text font-bold mb-2">Shader Playground</h2>
          <p class="mb-6 text-text">
            Tweak any preset or pick another demo below.
          </p>

          <ul id="demo-tabs" class="flex gap-4 mb-4 border-b text-text">
            <li><button data-demo="orbit"  class="tab active">Orbit Path</button></li>
            <li><button data-demo="plasma" class="tab">Plasma Exhaust</button></li>
            <li><button data-demo="noise"  class="tab">Wobbly Planet</button></li>
          </ul>

          <div class="grid md:grid-cols-2 gap-6">
            <textarea id="shader" class="w-full h-128 bg-surface text-text p-4 font-mono
                                         rounded-xl resize-none"></textarea>

          // TODO: make demo's cover full width and have the editor be a tab you can switch to
          // instead of one next to the other as the demo window on mobile is too small
            <div class="w-full h-128 rounded-xl border overflow-hidden flex items-center justify-center">
              <CubeDemo />
            </div>
          </div>

          <p class="mt-4 text-sm text-text">
            More doodles live at <a href="/projects/experiments" class="underline">experiments</a>.
          </p>
        </section>
      </main>
    }
}
