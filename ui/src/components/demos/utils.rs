use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use anyhow::anyhow;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::ClassAttribute;
use leptos::prelude::Effect;
use leptos::prelude::ElementChild;
use leptos::prelude::GetUntracked;
use leptos::prelude::RwSignal;
use leptos::prelude::Set;
use leptos::reactive::spawn_local;
use leptos::{IntoView, component, view};
use wasm_bindgen::{JsCast, convert::FromWasmAbi, prelude::Closure};
use web_sys::HtmlCanvasElement;

use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::gpu_state::Projection;
use crate::render::renderer::gpu::GpuState;
use crate::render::renderer::mesh::CpuMesh;
use crate::render::web_gpu::init_wgpu;
use crate::render::web_gpu::reload_pipeline;

/// true on PCs with a mouse/track-pad, false on touch devices
pub fn is_desktop() -> bool {
    let win = web_sys::window().unwrap();
    // ① media query: fine pointer ⇒ mouse/trackpad
    if let Ok(Some(mql)) = win.match_media("(pointer: fine)") {
        if mql.matches() { return true; }
    }
    // ② fallback: no touch points ⇒ desktop
    win.navigator().max_touch_points() == 0
}

#[component]
pub fn WebGPUNotSupportedMsg() -> impl IntoView {
    view! {
        <div class="max-w-md mx-auto bg-blue-50 border border-blue-200 rounded-lg p-6 shadow-sm text-blue-800">
          <h2 class="text-xl font-semibold mb-2">WebGPU Not Supported</h2>
          <p class="mb-4 leading-relaxed">
            This demo uses <span class="font-bold">WebGPU</span> for next-gen graphics.<br/>
            Your browser does not support WebGPU yet.
          </p>
          <p class="font-semibold mb-3">To try the demos, use one of these browsers:</p>
          <div class="flex flex-wrap gap-2 mb-4">
            <a href="https://www.google.com/chrome/" target="_blank" rel="noopener"
               class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full hover:bg-blue-200 transition">
              Chrome
            </a>
            <a href="https://www.microsoft.com/edge" target="_blank" rel="noopener"
               class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full hover:bg-blue-200 transition">
              Edge
            </a>
            <a href="https://www.opera.com/" target="_blank" rel="noopener"
               class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full hover:bg-blue-200 transition">
              Opera
            </a>
            <a href="https://www.chromium.org/getting-involved/download-chromium/" target="_blank" rel="noopener"
               class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full hover:bg-blue-200 transition">
              Chromium
            </a>
          </div>
          <p class="text-sm text-blue-700">
            <span class="font-semibold">Note:</span>"Firefox and Safari do "<i>not</i> "support WebGPU by default as of 2025."<br/>
            Enable <i>"chrome://flags/#enable-unsafe-webgpu"</i>" in Chrome if needed."
          </p>
        </div>
    }
}

pub fn add_listener<T, F>(target: &HtmlCanvasElement, ty: &str, f: F)
where
    T: 'static + JsCast + FromWasmAbi,
    F: 'static + FnMut(T),
{
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut(_)>);
    target
        .add_event_listener_with_callback(ty, cb.as_ref().unchecked_ref())
        .unwrap();
    cb.forget();
}

pub fn add_camera_orbit(
    camera_input: &Rc<RefCell<Option<CameraInput>>>,
    canvas: &HtmlCanvasElement,
    show_hint: RwSignal<bool>,
) -> Result<()> {
    if camera_input.clone().borrow().is_none() {
        return Err(anyhow!("Gpu state is None"));
    }

    assert!(
        camera_input.clone().borrow().is_some(),
        "GpuState is None and somehow passed the guard clause. This should not be possible."
    ); // now I can safely unwrap past this point

    let cv = canvas.clone();
    let ci = camera_input.clone();
    add_listener(&canvas, "pointerdown", move |e: web_sys::PointerEvent| {
        if e.button() != 0 {
            return;
        }

        if show_hint.get_untracked() {
            show_hint.set(false);
        }

        let _ = cv.set_pointer_capture(e.pointer_id());

        if let Ok(mut guard) = ci.try_borrow_mut() {
            let ci = guard.as_mut().unwrap();
            ci.dragging = true;

            let w = cv.width() as f32;
            let h = cv.height() as f32;

            // record starting mouse position (canvas‐relative)
            let mx = (e.client_x() as f32) - w;
            let my = (e.client_y() as f32) - h;
            ci.last_mouse_pos = (mx, my);
        }

        // prevent default so canvas doesn’t lose focus
        e.prevent_default();
    });

    // ─── MOUSEMOVE ───
    let ci = camera_input.clone();
    let cv = canvas.clone();
    add_listener(&canvas, "pointermove", move |e: web_sys::PointerEvent| {
        if let Ok(mut guard) = ci.try_borrow_mut() {
            let ci = guard.as_mut().unwrap();

            if !ci.dragging {
                return;
            }

            let w = cv.width() as f32;
            let h = cv.height() as f32;

            // compute delta since last frame
            let mx = (e.client_x() as f32) - w;
            let my = (e.client_y() as f32) - h;

            let (lx, ly) = ci.last_mouse_pos;
            let dx = mx - lx;
            let dy = my - ly;
            ci.last_mouse_pos = (mx, my);

            // update camera angles
            ci.camera.yaw += dx * 0.005;
            ci.camera.pitch += dy * 0.005;

            // clamp pitch so we don’t flip upside‐down:
            let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
            ci.camera.pitch = ci.camera.pitch.clamp(-max_pitch, max_pitch);
        }

        e.prevent_default();
    });

    // ─── MOUSEUP / MOUSELEAVE ───
    let ci = camera_input.clone();
    let cv = canvas.clone();
    add_listener(&canvas, "pointerup", move |e: web_sys::PointerEvent| {
        let _ = cv.release_pointer_capture(e.pointer_id());

        if let Ok(mut guard) = ci.try_borrow_mut() {
            let ci = guard.as_mut().unwrap();
            ci.dragging = false;
        }
    });

    let ci = camera_input.clone();
    add_listener(&canvas, "pointerleave", move |_: web_sys::PointerEvent| {
        if let Ok(mut guard) = ci.try_borrow_mut() {
            let ci = guard.as_mut().unwrap();
            ci.dragging = false;
        }
    });

    Ok(())
}

pub fn add_mousewheel_zoom(camera_input: &Rc<RefCell<Option<CameraInput>>>, canvas: &HtmlCanvasElement) -> Result<()> {
    if camera_input.clone().borrow().is_none() {
        return Err(anyhow!("Gpu state is None"));
    }

    assert!(
        camera_input.clone().borrow().is_some(),
        "GpuState is None and somehow passed the guard clause. This should not be possible."
    ); // now I can safely unwrap past this point


    let ci = camera_input.clone();
    add_listener(&canvas, "wheel", move |e: web_sys::WheelEvent| {
        if let Ok(mut guard) = ci.try_borrow_mut() {
            let ci = guard.as_mut().unwrap();
            let delta = e.delta_y() as f32 * 0.01;
            ci.camera.distance = (ci.camera.distance + delta).clamp(1.0, 50.0);
        }
        e.prevent_default();
    });

    Ok(())
}

pub fn start_rendering(
    state_rc: Rc<RefCell<Option<GpuState>>>,
    camera_rc: Rc<RefCell<Option<CameraInput>>>,

    show_hint: RwSignal<bool>,
    gpu_support: RwSignal<bool>,
    pending: RwSignal<Option<(String, String)>>,

    canvas_id: &str,

    mesh: Rc<RefCell<CpuMesh<'static>>>,
    projection: Rc<RefCell<Projection>>,
) {
        let state_for_init = state_rc.clone();
        let show_hint_for_init = show_hint.clone();
        let canvas_id = canvas_id.to_string();
        let camera_rc = camera_rc.clone();

        Effect::new(move |_| {
            let state_for_spawn = state_for_init.clone();
            let show_hint = show_hint_for_init.clone();
            let id = canvas_id.clone();
            let camera_for_spawn = camera_rc.clone();
            let mesh_rc = mesh.clone();
            let proj_rc = projection.clone();

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

                let state_rc = state_for_spawn.clone();
                *state_rc.borrow_mut() = Some(state);

                let camera_rc = camera_for_spawn.clone();
                *camera_rc.borrow_mut() = Some(CameraInput::default());

                if let Err(e) = add_camera_orbit(&camera_rc, &canvas, show_hint) {
                    web_sys::console::error_1(&format!("add_camera_orbit failed: {e:?}").into());
                }
                if let Err(e) = add_mousewheel_zoom(&camera_rc, &canvas) {
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
                                s.render_default(
                                    &proj_rc.borrow(),
                                    ci,
                                    &mesh_rc.borrow()
                                );
                            }
                        }
                    }

                    // 2) schedule next frame
                    web_sys::window()
                        .unwrap()
                        .request_animation_frame(
                            f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                        )
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
