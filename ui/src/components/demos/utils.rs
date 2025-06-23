use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use anyhow::anyhow;
use gloo_timers::future::TimeoutFuture;
use leptos::leptos_dom::logging::console_log;
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
use wgpu::RenderPipeline;

use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::GpuState;
use crate::render::renderer::gpu::gpu_state::FrameCtx;
use crate::render::web_gpu::init_wgpu;
use crate::render::web_gpu::reload_pipeline;

/// true on PCs with a mouse/track-pad, false on touch devices
pub fn is_desktop() -> bool {
    let win = web_sys::window().unwrap();
    if let Ok(Some(mql)) = win.match_media("(pointer: fine)") {
        if mql.matches() {
            return true;
        }
    }
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

pub fn add_mousewheel_zoom(
    camera_input: &Rc<RefCell<Option<CameraInput>>>,
    canvas: &HtmlCanvasElement,
) -> Result<()> {
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

fn get_canvas(id: &str) -> Option<HtmlCanvasElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id(id)?
        .dyn_into::<HtmlCanvasElement>()
        .ok()
}

fn add_input_handlers(
    camera: &Rc<RefCell<Option<CameraInput>>>,
    canvas: &HtmlCanvasElement,
    show_hint: RwSignal<bool>,
) {
    if let Err(e) = add_camera_orbit(camera, canvas, show_hint) {
        web_sys::console::error_1(&format!("add_camera_orbit failed: {e:?}").into());
    }
    if let Err(e) = add_mousewheel_zoom(camera, canvas) {
        web_sys::console::error_1(&format!("add_mousewheel_zoom failed: {e:?}").into());
    }
}

pub type RenderPass = Rc<RefCell<dyn FnMut(&mut GpuState, &CameraInput, &mut FrameCtx)>>;

fn start_render_loop(
    state_rc: Rc<RefCell<Option<GpuState>>>,
    camera_rc: Rc<RefCell<Option<CameraInput>>>,

    rpasses: Vec<RenderPass>,

    mut on_frame: impl 'static + FnMut(),
) {
    let raf: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let raf_clone = raf.clone();

    *raf_clone.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
        on_frame();

        // ── Run user passes --------------------------------------------------
        if let (Some(state), Ok(cam_ref)) = (state_rc.borrow_mut().as_mut(), camera_rc.try_borrow())
        {
            let cam = cam_ref.as_ref().expect("CameraInput is None");

            let mut ctx = state.begin_frame();

            for pass in &rpasses {
                (pass.borrow_mut())(state, cam, &mut ctx);
            }

            state.end_frame(ctx);
        }

        // ── Next frame -------------------------------------------------------
        web_sys::window()
            .unwrap()
            .request_animation_frame(raf.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut(f64)>));

    web_sys::window()
        .unwrap()
        .request_animation_frame(
            raf_clone
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();
}

pub fn start_rendering<F, G>(
    state_rc: Rc<RefCell<Option<GpuState>>>,
    camera_rc: Rc<RefCell<Option<CameraInput>>>,

    show_hint: RwSignal<bool>,
    gpu_support: RwSignal<bool>,
    pending: RwSignal<Option<(String, String)>>,

    canvas_id: &str,

    rpasses: Vec<RenderPass>,
    pipes: Vec<Rc<RefCell<Option<wgpu::RenderPipeline>>>>,

    on_canvas_ready: F, // extra closure after canvas is ready hook
    on_frame: G,        // extra closure to run every frame
) where
    F: 'static + Fn(&HtmlCanvasElement) + Clone,
    G: 'static + FnMut() + Clone,
{
    let canvas_id = canvas_id.to_owned();
    let on_canvas_ready = Rc::new(on_canvas_ready);
    let on_frame = on_frame.clone();

    Effect::new(move |_| {
        let state_rc_init = state_rc.clone();
        let camera_rc_init = camera_rc.clone();

        let show_hint = show_hint.clone();
        let gpu_support = gpu_support.clone();

        let canvas_id = canvas_id.clone();

        let rpasses_init = rpasses.clone();

        let on_ready = on_canvas_ready.clone();
        let on_frame = on_frame.clone();

        spawn_local(async move {
            TimeoutFuture::new(0).await;

            let canvas = match get_canvas(&canvas_id) {
                Some(c) => c,
                None => {
                    web_sys::console::error_1(&"Canvas not found".into());
                    return;
                }
            };

            // run user hook
            (on_ready)(&canvas);

            let state = match init_wgpu(&canvas).await {
                Ok(s) => s,
                Err(err) => {
                    gpu_support.set(false);
                    web_sys::console::error_1(&format!("WGPU init failed: {err:?}").into());
                    return;
                }
            };

            *state_rc_init.borrow_mut() = Some(state);
            *camera_rc_init.borrow_mut() = Some(CameraInput::default());

            add_input_handlers(&camera_rc_init, &canvas, show_hint);

            start_render_loop(
                state_rc_init,
                camera_rc_init,
                rpasses_init,
                on_frame,
            );
        });
    });
}
