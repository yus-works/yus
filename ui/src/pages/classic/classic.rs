use gloo_timers::callback::Timeout;
use leptos::html;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::AriaAttributes;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::For;
use leptos::prelude::GlobalAttributes;
use leptos::prelude::IntoAny;
use leptos::prelude::NodeRef;
use leptos::prelude::NodeRefAttribute;
use leptos::prelude::OnAttribute;
use leptos::prelude::RwSignal;
use leptos::prelude::StyleAttribute;
use leptos::prelude::Suspense;
use leptos::prelude::Update;
use leptos::prelude::view;
use leptos::prelude::{Children, Effect, Get, Set};
use leptos::server::LocalResource;
use leptos::{IntoView, component};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::AddEventListenerOptions;
use web_sys::Element;
use web_sys::Event;

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
fn LangsTooltip(dto: ProjectDto) -> impl IntoView {
    let tooltip_lines = dto
        .languages
        .iter()
        .map(|l| format!("{} {:.1}%", l.name, l.pct))
        .collect::<Vec<_>>();

    view! {
        <div class="
            pointer-events-none
            absolute bottom-full left-1/2 -translate-x-1/2 mb-2
            hidden group-hover:block
        "
        aria-hidden="true">
            <div class="bg-neutral-900 text-xs text-white px-3 py-2 rounded shadow">
                <ul class="space-y-0.5">
                    <For
                        each=move || tooltip_lines.clone()
                        key=|s| s.clone()
                        children=move |line| {
                            view! { <li>{ line }</li> }
                        }
                    />
                </ul>
            </div>
        </div>
    }
}

#[component]
fn LangsLine(dto: ProjectDto) -> impl IntoView {
    let style = move |seg: LangDto| {
        format!(
            "flex:{:.3} 0 0;background:{};",
            seg.pct, seg.color.as_deref().unwrap_or("#666")
        )
    };


    let icon = move |seg: LangDto, show_icon: bool| {
        if show_icon {
            seg.icon.as_ref().map(|ic| view!{
                <i class=format!("devicon-{} text-white text-[1.25rem] leading-none", ic)></i>
            })
        } else { None }
    };

    let children = move |seg: LangDto| {
        let s = seg.clone();
        let show_icon = seg.pct >= 8.0;
        view! {
            <div
                class="basis-0 relative flex items-center justify-center"
                style=style(s)
            >
                { icon(seg, show_icon) }
            </div>
        }
    };

    view! {
        <div class="relative w-full h-8 rounded-b-xl overflow-hidden flex">
            <For
                each=move || dto.languages.clone()
                key=|seg| seg.name.clone()
                children=children
            />
        </div>
    }
}

#[component]
fn ProjectCard(
    dto: ProjectDto,
    image: String,
    #[prop(optional)] extra: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <article class="relative bg-neutral-light rounded-xl overflow-hidden shadow flex-shrink-0 w-80 snap-start flex flex-col">
            { children() }
            <img src=image alt="No image here yet :o" class="pt-8 h-40 w-full object-cover"/>
            <div class="p-4">
                <h3 class="font-semibold text-lg mb-1">{ dto.name.clone() }</h3>
                <p class="text-sm">
                    { dto.description.clone() }
                    { move || extra.map(|e| view! { <br/> <span>{ e }</span> } ) }
                </p>
            </div>
            <div class="group relative w-full mt-3 mt-auto">
                <LangsTooltip dto=dto.clone() />
                <LangsLine dto=dto.clone() />
            </div>
        </article>
    }
}

use gloo_net::http::Request;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct LangDto {
    name: String,
    pct: f32,
    color: Option<String>,
    icon: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ProjectDto {
    name: String,
    description: Option<String>,
    version: Option<String>,
    status: String,
    labels: Vec<String>,     // from repo topics
    languages: Vec<LangDto>, // (lang, pct)
}

#[component]
pub fn CarouselDots(selected: RwSignal<usize>, total: usize) -> impl IntoView {
    view! {
        <div class="flex justify-center gap-2 pt-4 select-none">
            <For
                each=move || 0..total
                key=|i| i + 1
                children=move |i| {
                    let selected = selected.clone();
                    view! {
                        <button
                            class=move || format!(
                                "h-2 w-2 rounded-full transition-all \
                                 duration-300 {}",
                                if selected.get() == i {
                                    "bg-primary/90 scale-125"
                                } else {
                                    "bg-gray-400/70 hover:bg-gray-500"
                                }
                            )
                            // keep buttons keyboard-focusable
                            aria-label=format!("Go to slide {}", i + 1)
                            on:click=move |_| selected.set(i)
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn ProjectCards() -> impl IntoView {
    let projects = LocalResource::new(|| async {
        Request::get("/api/projects")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<ProjectDto>>()
            .await
            .map_err(|e| e.to_string())
    });

    let card = move |p: ProjectDto| {
        let badge_bg = match p.status.as_str() {
            "active" => "bg-green-600/90",
            "ongoing" => "bg-orange-600/90",
            "paused" => "bg-yellow-600/90",
            _ => "bg-slate-600/90",
        };

        view! {
            <ProjectCard
                dto=p.clone()
                image=String::from("/img/rocket.jpg")
            >
                <span
                    class=format!(
                        "absolute top-2 left-2 text-xs px-2 py-0.5 rounded-full \
                         {} text-text", badge_bg)
                >
                    { move || p.status.clone() }
                </span>
            </ProjectCard>
        }
        .into_any()
    };

    let selected = RwSignal::new(0usize);
    let lane_ref: NodeRef<html::Div> = NodeRef::new();

    let autoscrolling = RwSignal::new(false);

    Effect::new(move |_| {
        let i = selected.get();
        if let (Some(lane), Some(child)) = (
            lane_ref.get(),
            lane_ref.get().and_then(|l| l.children().item(i as u32)),
        ) {
            autoscrolling.set(true);

            let opts = web_sys::ScrollIntoViewOptions::new();
            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
            opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
            opts.set_inline(web_sys::ScrollLogicalPosition::Start);
            child
                .unchecked_into::<Element>()
                .scroll_into_view_with_scroll_into_view_options(&opts);

            {
                let autoscrolling_flag = autoscrolling.clone();
                let cb = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_| {
                    autoscrolling_flag.set(false);
                }));

                let opts = AddEventListenerOptions::new();
                opts.set_once(true); // auto-remove after first fire
                opts.set_capture(true); // capture phase = fire sooner/always

                if lane
                    .add_event_listener_with_callback_and_add_event_listener_options(
                        "scrollend",
                        cb.as_ref().unchecked_ref(),
                        &opts,
                    )
                    .is_ok()
                {
                    cb.forget();
                } else {
                    // fallback
                    Timeout::new(400, move || autoscrolling.set(false)).forget();
                }
            }
        }
    });

    let on_scroll = move |_| {
        if autoscrolling.get() {
            console_log("autoscrolling gating");
            return;
        }
        console_log("autoscrolling not gating");
        if let Some(lane) = lane_ref.get() {
            let scroll_left = lane.scroll_left() as f32;
            let card_w = 320.0;
            let gap = 32.0;
            let i = (scroll_left / (card_w + gap)).round() as usize;

            if i != selected.get() {
                selected.set(i);
            }
        }
    };

    view! {
        <section id="projects" class="py-16">
            <Suspense fallback=|| view!{ <p class="text-text">"loading…"</p> } >
                { move || match projects.get() {
                    Some(Ok(list)) => {
                        let total = list.len();
                        view! {
                            <div
                                node_ref=lane_ref
                                class="flex gap-8 pb-3 overflow-x-auto snap-x snap-mandatory scroll-smooth"
                                on:scroll=on_scroll
                            >
                                <For
                                    each = move || list.clone()
                                    key = |p: &ProjectDto| p.name.clone()
                                    children = move |p| { card(p) }
                                />
                            </div>

                            <div class="mt-8">
                                <CarouselDots selected total />
                            </div>
                        }
                    }.into_any(),

                    Some(Err(e)) => view!{ <p class="text-red-500">"error: " {e}</p> }.into_any(),
                    None => ().into_any(),
                }}
            </Suspense>
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
        self.0.update(|m| {
            m.insert(label.into(), sig);
        });
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

            <p class="text-text leading-relaxed">
                "Needed a cube-planet visualiser for my Minecraft space-mod "
                <a href="github.com/yus-works/velari" class="underline hover:text-accent">Velari</a>". "
                "I picked up "<strong>wgpu</strong>", liked it, figured out how to integrate it with Leptos, and so here's a bunch of graphics demos I made for fun."<br/>
            </p>

            <br/>

            <ul id="demo-tabs" class="flex gap-4 mb-4 border-b text-text">
                {Demo::Animals     .labelled_button(selected_demo)}
                {Demo::CubePlanet  .labelled_button(selected_demo)}
                {Demo::FragIntro   .labelled_button(selected_demo)}
            </ul>

            { move || selected_demo.get().description() }

            <div class="
                grid grid-cols-1 lg:grid-cols-2
                gap-y-12
                lg:gap-y-0
                lg:gap-x-6
            ">
                <ShaderEditor vs_src fs_src pass_flags=pass_flags.clone() selected_demo />

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
          <ShaderLab/>
      </main>
    }
}
