use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use anyhow::anyhow;
use leptos::prelude::ClassAttribute;
use leptos::prelude::ElementChild;
use leptos::prelude::GetUntracked;
use leptos::prelude::RwSignal;
use leptos::prelude::Set;
use leptos::{IntoView, component, view};
use wasm_bindgen::{JsCast, convert::FromWasmAbi, prelude::Closure};
use web_sys::HtmlCanvasElement;

use crate::render::renderer::camera_input;
use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::GpuState;

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
