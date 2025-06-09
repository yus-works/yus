use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GlobalAttributes, RwSignal, Set, Show,
};
use std::cell::RefCell;
use std::rc::Rc;

use leptos::view;

use crate::render::renderer::gpu::GpuState;
use crate::render::web_gpu::init_wgpu;
use crate::web_sys::HtmlCanvasElement;
use leptos::IntoView;
use leptos::component;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use super::utils;
use web_sys;

use gloo_timers::future::TimeoutFuture;

#[component]
pub fn CubeDemo() -> impl IntoView {
    let canvas_id = "cube-demo-canvas";

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    // runs once “next tick” of Leptos
    Effect::new(move |_| {
        let id = canvas_id.to_string();
        spawn_local(async move {
            // 1) wait until the <canvas> actually exists
            TimeoutFuture::new(0).await;

            // 2) grab the DOM canvas
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas: HtmlCanvasElement = document
                .get_element_by_id(&id)
                .expect("canvas not in DOM yet")
                .dyn_into::<HtmlCanvasElement>()
                .expect("element is not a canvas");

            // 3) init WGPU with that canvas
            let state = match init_wgpu(&canvas).await {
                Ok(s) => s,
                Err(err) => {
                    gpu_support.set(false);
                    web_sys::console::error_1(&format!("WGPU init failed: {:?}", err).into());
                    return;
                }
            };

            // 4) wrap state in Rc<RefCell> so event closures can mutate it
            let state_rc: Rc<RefCell<GpuState>> = Rc::new(RefCell::new(state));

            utils::add_camera_orbit(&state_rc, &canvas, show_hint);
            utils::add_mousewheel_zoom(&state_rc, &canvas);

            // ───  RENDER LOOP ────────────────────────────────────────────────────────────────

            // we’ll store the RAF callback so we can re‐schedule it each frame
            let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();

            // now we kick off requestAnimationFrame(…) and draw each frame:
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
                // 1) borrow‐and‐render one frame
                {
                    let mut s = state_rc.borrow_mut();
                    s.render();
                }

                // 2) schedule next frame
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .unwrap();
            }) as Box<dyn FnMut(f64)>));

            // initial kick
            web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .unwrap();
        });
    });

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
            height="600"
            class="w-full"
          ></canvas>

          <Show when=move || show_hint.get()>
              <div id="hint"
                   class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center
                          bg-white/70 backdrop-blur-sm text-surface text-sm gap-2
                          transition-opacity duration-500
                          group-hover:opacity-0">
                "✋"
                <p>"Click & drag – scroll to zoom"</p>
              </div>
          </Show>

          </Show>
        </div>
    }
}
