use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GlobalAttributes, RwSignal, Set, Show,
};
use std::cell::RefCell;
use std::rc::Rc;

use leptos::view;

use crate::meshes;
use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::gpu_state::make_default_rpass;
use crate::render::renderer::gpu::gpu_state::Projection;
use crate::render::renderer::gpu::GpuState;
use crate::render::renderer::mesh::CpuMesh;
use leptos::IntoView;
use leptos::component;
use super::utils;


#[component]
pub fn CubePlanet(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let canvas_id = "cube-demo-canvas";

    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));
    let pending = RwSignal::new(None::<(String, String)>);

    let camera_rc: Rc<RefCell<Option<CameraInput>>> = Rc::new(RefCell::new(None));

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    {
        let pending = pending.clone();
        Effect::new(move |_| {
            pending.set(Some((vs_src.get(), fs_src.get())));
        });
    }

    let mesh = CpuMesh::new(
        meshes::cube::CUBE_VERTICES.to_vec(),
        meshes::cube::CUBE_INDICES.to_vec(),
    );

    let mesh = Rc::new(RefCell::new(mesh));

    let proj = Rc::new(RefCell::new(Projection::Fulcrum));

    utils::start_rendering(
        state_rc, camera_rc,
        show_hint, gpu_support,
        pending, canvas_id,
        vec![make_default_rpass(mesh, proj)],
        vec![],
        |_| {},
        || {},
    );

    // 5) return the <canvas> in the view – Leptos mounts it, then our Effect hooks it.
    view! {
        <div class="relative w-full group">
          <Show
            when=move || matches!(gpu_support.get(), true)
            fallback=move || view! { <utils::WebGPUNotSupportedMsg/> }
          >

          <canvas
            id=canvas_id
            width="800"
            height="800"
            class="w-full h-full object-cover touch-none select-none"
          ></canvas>

          <Show when=move || show_hint.get()>
              <div id="hint"
                   class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center
                          bg-white/70 backdrop-blur-sm text-surface text-sm gap-2
                          transition-opacity duration-500
                          group-hover:opacity-0">
                ""
                <p>"Click & drag to rotate camera ✋"</p>
                <p>"Scroll or pinch to zoom 🖱️/🤏"</p>
                <strong>"Click to hide this hint"</strong>
              </div>
          </Show>

          </Show>
        </div>
    }
}
