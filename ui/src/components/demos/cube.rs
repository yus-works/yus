use gloo_timers::callback::Timeout;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GetUntracked, GlobalAttributes, RwSignal, Set, Show
};
use wgpu::wgc::device::queue;
use std::cell::RefCell;
use std::rc::Rc;

use leptos::view;

use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::utils::FragmentShader;
use crate::render::renderer::gpu::utils::VertexShader;
use crate::render::renderer::gpu::GpuState;
use crate::render::web_gpu::{self, reload_pipeline};
use crate::render::web_gpu::init_wgpu;
use crate::web_sys::HtmlCanvasElement;
use leptos::IntoView;
use leptos::component;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen_futures::spawn_local;

use super::utils;
use web_sys;

use gloo_timers::future::TimeoutFuture;

#[component]
pub fn CubeDemo(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let canvas_id = "cube-demo-canvas";

    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));
    let pending = RwSignal::new(None::<(String,String)>);

    let camera_rc: Rc<RefCell<Option<CameraInput>>> = Rc::new(RefCell::new(None));

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    {
        let pending = pending.clone();
        Effect::new(move |_| {
            pending.set(Some((vs_src.get(), fs_src.get())));
        });
    }

    {
        let state_for_init = state_rc.clone();
        let show_hint_for_init = show_hint.clone();
        let canvas_id = canvas_id.to_string();
        let camera_rc = camera_rc.clone();

        Effect::new(move |_| {
            let state_for_spawn = state_for_init.clone();
            let show_hint = show_hint_for_init.clone();
            let id = canvas_id.clone();
            let camera_for_spawn = camera_rc.clone();

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

                let state_rc = state_for_spawn.clone();          // ← use the outer Rc
                *state_rc.borrow_mut() = Some(state);      //   put the real state inside

                let camera_rc = camera_for_spawn.clone();
                *camera_rc.borrow_mut() = Some(CameraInput::default());

                if let Err(e) = utils::add_camera_orbit(&camera_rc, &canvas, show_hint) {
                    web_sys::console::error_1(&format!("add_camera_orbit failed: {e:?}").into());
                }
                if let Err(e) = utils::add_mousewheel_zoom(&camera_rc, &canvas) {
                    web_sys::console::error_1(&format!("add_mousewheel_zoom failed: {e:?}").into());
                }

                // ───  RENDER LOOP ────────────────────────────────────────────────────────────────

                // we’ll store the RAF callback so we can re‐schedule it each frame
                let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
                let g = f.clone();

                // now we kick off requestAnimationFrame(…) and draw each frame:
                *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {

                    // 1) borrow‐and‐render one frame
                    {
                        if state_rc.clone().borrow().is_none() {
                            web_sys::console::error_1(&format!("State is somehow none?").into());
                        }

                        let st = state_rc.clone();

                        if let Some((vs, fs)) = pending.get_untracked() {
                            spawn_local(async move {
                                if let Err(msg) = reload_pipeline(&st, &vs, &fs).await {
                                    // TODO: show failed to compile error to user
                                    web_sys::console::error_1(&msg.to_string().into());
                                }
                            });

                            pending.set(None);
                        }

                        let mut guard = state_rc.borrow_mut();
                        let s = guard.as_mut().unwrap();

                        if let Ok(ci_ref) = camera_rc.try_borrow() {
                            if let Some(ci) = &*ci_ref {
                                s.render(ci);
                            }
                        }
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
    }

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
            class="w-full h-full object-cover touch-none select-none"
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
