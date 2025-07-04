use std::collections::HashMap;
use leptos::prelude::ClassAttribute;
use leptos::prelude::{Children, Effect, Get, Set};
use leptos::prelude::ElementChild;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::RwSignal;
use leptos::prelude::view;
use leptos::prelude::Update;
use leptos::{IntoView, component};

use crate::components::demo::{Demo, DemoTab};
use crate::components::shader_editor::ShaderEditor;

#[component]
fn Hero() -> impl IntoView {
    view! {
        <section class="py-24">
            <h1 class="text-5xl sm:text-7xl font-display text-text mb-6 leading-tight">
            Curious by nature,<br/>Serious by habit.
            </h1>

            <a
                href="#projects"
                class="inline-block bg-primary text-neutral-dark px-8 py-4 rounded-full hover:bg-primary transition"
            >View Projects</a>
        </section>
    }
}

#[component]
pub fn ProjectCard(
    title: &'static str,
    desc: &'static str,
    image: &'static str,
    #[prop(optional)]
    extra: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <article class="relative bg-neutral-light rounded-xl overflow-hidden shadow flex-shrink-0 w-80 snap-start">
            { children() }
            <img src=image alt=title class="h-40 w-full object-cover"/>
            <div class="p-4">
                <h3 class="font-semibold text-lg mb-1">{ title }</h3>
                <p class="text-sm text-slate-700">
                    { desc }
                    { move || extra.map(|e| view! { <br/> <span>{ e }</span> } ) }
                </p>
            </div>
        </article>
    }
}

#[component]
pub fn ProjectCards() -> impl IntoView {
    view! {
        <section id="projects" class="py-16 flex gap-8 overflow-x-auto snap-x snap-mandatory scrollbar-hidden">
            <ProjectCard
                title="yus.rs"
                desc="Portfolio/Personal Brand"
                image="/img/rocket.jpg"
            >
                <span class="absolute top-2 left-2 bg-green-600/90 text-text text-xs px-2 py-0.5 rounded-full">"Live"</span>
            </ProjectCard>
            <ProjectCard
                title="Velari"
                desc="Minecraft Space Travel Mod"
                image="/img/rocket.jpg"
                extra="GPL-3.0"
            >
                <span class="absolute top-2 left-2 bg-green-600/90 text-text text-xs px-2 py-0.5 rounded-full">"WIP"</span>
            </ProjectCard>

            <ProjectCard
                title="Metal Stars"
                desc="AR Satellite Visualizer"
                image="/img/rocket.jpg"
                extra="GPL-3.0"
            >
                <span class="absolute top-2 left-2 bg-green-600/90 text-text text-xs px-2 py-0.5 rounded-full">"WIP"</span>
            </ProjectCard>

            <ProjectCard
                title="Yus Experiments"
                desc="Experiments that test my abilities"
                image="/img/rocket.jpg"
            >
                <span class="absolute top-2 left-2 bg-slate-600/90 text-text text-xs px-2 py-0.5 rounded-full">"Experiments"</span>
            </ProjectCard>

            <ProjectCard
                title="HeliOS"
                desc="Experiments that test my abilities"
                image="/img/rocket.jpg"
            >
                <span class="absolute top-2 left-2 bg-slate-600/90 text-text text-xs px-2 py-0.5 rounded-full">"Experiments"</span>
            </ProjectCard>

            <ProjectCard
                title="Nebula"
                desc="Experiments that test my abilities"
                image="/img/rocket.jpg"
            >
                <span class="absolute top-2 left-2 bg-slate-600/90 text-text text-xs px-2 py-0.5 rounded-full">"Experiments"</span>
            </ProjectCard>

            <ProjectCard
                title="Plantorio"
                desc="Experiments that test my abilities"
                image="/img/rocket.jpg"
            >
                <span class="absolute top-2 left-2 bg-slate-600/90 text-text text-xs px-2 py-0.5 rounded-full">"Experiments"</span>
            </ProjectCard>
        </section>
    }
}

// Tailwind helper: hide scrollbars without disabling scrolling.
// Add this to your CSS (or Tailwind plugin):
// .scrollbar-hidden { scrollbar-width: none; }
// .scrollbar-hidden::-webkit-scrollbar { display: none; }

#[component]
fn Experiments() -> impl IntoView {
    view! {
        <section id="experiments" class="py-8">
            <h2 class="text-3xl font-bold text-text">Yus Experiments</h2>

            <p class="text-text leading-relaxed">
                "Needed a cube-planet visualiser for my Minecraft space-mod "
                <a href="github.com/yus-works/velari" class="underline hover:text-[#E55934]">Velari</a>". "
                "I picked up "<strong>wgpu</strong>", liked it, figured out how to integrate it with Leptos, and so here's a bunch of graphics demos I made for fun."<br/>
            </p>
        </section>
    }
}

#[derive(Clone)]
pub struct PassFlags(RwSignal<HashMap<String, RwSignal<bool>>>);

impl PassFlags {
    pub fn new() -> Self {
        Self(RwSignal::new(HashMap::new()))
    }

    pub fn init_pass(&self, label: &str, state: bool) -> RwSignal<bool> {
        let sig = RwSignal::new(state);
        self.0.update(|m| { m.insert(label.into(), sig); });
        sig
    }

    pub fn iter(&self) -> Vec<(String, RwSignal<bool>)> {
        self.0
            .get() // reactive
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

#[component]
fn ShaderLab() -> impl IntoView {
    let selected_demo = RwSignal::new(Demo::Animals);
    let vs_src =
        RwSignal::new(include_str!("../../render/renderer/shaders/fish.vert.wgsl").to_owned());
    let fs_src =
        RwSignal::new(include_str!("../../render/renderer/shaders/fish.frag.wgsl").to_owned());

    // whenever demo changes, push its shader pair into the two text signals
    Effect::new(move |_| {
        let (vs, fs) = selected_demo.get().shaders();
        vs_src.set(vs.to_owned());
        fs_src.set(fs.to_owned());
    });

    let pass_flags = PassFlags::new();

    view! {
        <section id="shader-lab" class="py-8">
            <h2 class="text-3xl text-text font-bold mb-2">Shader Playground</h2>
            <p class="mb-6 text-text">
                Tweak any preset or pick another demo below.
            </p>

            <ul id="demo-tabs" class="flex gap-4 mb-4 border-b text-text">
                {Demo::Animals     .labelled_button(selected_demo)}
                {Demo::CubePlanet  .labelled_button(selected_demo)}
                {Demo::FragIntro   .labelled_button(selected_demo)}
            </ul>

            <div class="
                grid grid-cols-1 lg:grid-cols-2
                gap-y-12
                lg:gap-y-0
                lg:gap-x-6
            ">
                <ShaderEditor vs_src fs_src pass_flags=pass_flags.clone() selected_demo />

            // TODO: make demo's cover full width and have the editor be a tab you can switch to
            // instead of one next to the other as the demo window on mobile is too small
                <div class="w-full h-[40rem] rounded-xl border overflow-hidden flex items-center justify-center">
                    {
                        move || selected_demo.get().canvas(vs_src, fs_src, pass_flags.clone())
                    }
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn ClassicMain() -> impl IntoView {
    view! {
      <main class="max-w-6xl mx-auto px-6">
          <Hero/>
          <ProjectCards/>
          <Experiments/>
          <ShaderLab/>
      </main>
    }
}
