use glam::Vec2;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::OnAttribute;
use leptos::{
    IntoView,
    prelude::{AnyView, Get, IntoAny, RwSignal, Set},
    view,
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement, PointerEvent};

use crate::pages::classic::classic::PassFlags;

use super::demos::{animals::main::Animals, frag_intro::main::FragIntro, planet::main::CubePlanet};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Demo {
    CubePlanet,
    Animals,
    FragIntro,
}

const CUBE_VS: &str = include_str!("../render/renderer/shaders/cube.vert.wgsl");
const CUBE_FS: &str = include_str!("../render/renderer/shaders/cube.frag.wgsl");

const FRAG_VS: &str = include_str!("../render/renderer/shaders/frag_intro.vert.wgsl");
const FRAG_FS: &str = include_str!("../render/renderer/shaders/frag_intro.frag.wgsl");

const FISH_VS: &str = include_str!("../render/renderer/shaders/fish.vert.wgsl");
const FISH_FS: &str = include_str!("../render/renderer/shaders/fish.frag.wgsl");

impl Demo {
    pub fn label(&self) -> &'static str {
        match self {
            Demo::Animals => "Procedurally Animated Animals",
            Demo::CubePlanet => "Cube Planet Visualizer",
            Demo::FragIntro => "Fragment Shader Intro",
        }
    }
    pub fn shaders(&self) -> (&'static str, &'static str) {
        match self {
            Demo::Animals => (FISH_VS, FISH_FS),
            Demo::CubePlanet => (CUBE_VS, CUBE_FS),
            Demo::FragIntro => (FRAG_VS, FRAG_FS),
        }
    }

    /// Returns a View that mounts the proper canvas component
    pub fn canvas(
        self,
        vs_src: RwSignal<String>,
        fs_src: RwSignal<String>,
        pass_flags: PassFlags,
    ) -> AnyView {
        match self {
            Demo::Animals => view! { <Animals vs_src fs_src pass_flags/> }.into_any(),
            Demo::CubePlanet => view! { <CubePlanet vs_src fs_src/> }.into_any(),
            Demo::FragIntro => view! { <FragIntro vs_src fs_src/> }.into_any(),
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

pub fn to_clip_space(e: &PointerEvent, canvas: &HtmlCanvasElement) -> Vec2 {
    // canvas-relative position in **device pixels**
    let html: &HtmlElement = canvas.unchecked_ref();
    let rect = html.get_bounding_client_rect();
    let scale_x = canvas.width()  as f32 / rect.width()  as f32;   // Hi-DPI
    let scale_y = canvas.height() as f32 / rect.height() as f32;

    let x_px = (e.client_x() as f32 - rect.left() as f32) * scale_x;
    let y_px = (e.client_y() as f32 - rect.top()  as f32) * scale_y;

    // device-pixels → NDC
    let mut p = Vec2::new(
        2.0 * (x_px / canvas.width()  as f32) - 1.0,
       -2.0 * (y_px / canvas.height() as f32) + 1.0,
    );

    // reverse the squeeze that view_proj applies
    let aspect = canvas.width() as f32 / canvas.height() as f32;
    if aspect >= 1.0 {
        // shader shrinks x, so expand it back for the mouse
        p.x *= aspect;
    } else {
        // shader shrinks y, so expand it back for the mouse
        p.y /= aspect;
    }

    p
}
