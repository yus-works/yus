use leptos::{prelude::{AnyView, IntoAny, RwSignal, Get, Set}, view, IntoView};
use leptos::prelude::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::prelude::OnAttribute;

use super::demos::{animals::Animals, cube::CubePlanet};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Demo {
    CubePlanet,
    Animals,
}

const CUBE_VS: &str = include_str!("../render/renderer/shaders/cube.vert.wgsl");
const CUBE_FS: &str = include_str!("../render/renderer/shaders/cube.frag.wgsl");
const FISH_VS: &str = include_str!("../render/renderer/shaders/fish.vert.wgsl");
const FISH_FS: &str = include_str!("../render/renderer/shaders/fish.frag.wgsl");

impl Demo {
    pub fn label(&self) -> &'static str {
        match self {
            Demo::Animals => "Procedurally Animated Animals",
            Demo::CubePlanet => "Cube Planet Visualizer",
        }
    }
    pub fn shaders(&self) -> (&'static str, &'static str) {
        match self {
            Demo::Animals => (FISH_VS, FISH_FS),
            Demo::CubePlanet => (CUBE_VS, CUBE_FS),
        }
    }

    /// Returns a View that mounts the proper canvas component
    pub fn canvas(self, vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> AnyView {
        match self {
            Demo::Animals => view! { <Animals vs_src=vs_src fs_src=fs_src/> }.into_any(),
            Demo::CubePlanet => view! { <CubePlanet vs_src=vs_src fs_src=fs_src/> }.into_any(),
        }
    }
}

pub trait DemoTab {
    fn labelled_button(self, sel: RwSignal<Demo>) -> impl IntoView;
}

impl DemoTab for Demo {
    fn labelled_button(self, sel: RwSignal<Demo>) -> impl IntoView {
        view! {
            <li>
                <button
                    class="tab px-3 py-1"
                    // “active” class toggles styling
                    class=("active",  move || sel.get() == self)
                    // update selection on click
                    on:click=move |_| sel.set(self)
                >
                    {self.label()}
                </button>
            </li>
        }
    }
}
