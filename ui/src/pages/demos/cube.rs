use std::cell::RefCell;
use std::rc::Rc;

use leptos::attr::HttpEquiv;
use leptos::prelude::Effect;
use leptos::prelude::StyleAttribute;
use leptos::view;
use leptos::prelude::ClassAttribute;
use leptos::prelude::GlobalAttributes;

use leptos::component;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use crate::render::web_gpu::GpuState;
use crate::web_sys::HtmlCanvasElement;
use leptos::IntoView;
use wasm_bindgen_futures::spawn_local;
use crate::render::web_gpu::init_wgpu;

use web_sys;

use gloo_timers::future::TimeoutFuture;

#[component]
pub fn CubeDemo() -> impl IntoView {
    let canvas_id = "cube-demo-canvas";

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
                    web_sys::console::error_1(&format!("WGPU init failed: {:?}", err).into());
                    return;
                }
            };

            // 4) wrap state in Rc<RefCell> so event closures can mutate it
            let state_rc: Rc<RefCell<GpuState>> = Rc::new(RefCell::new(state));

            //
            // ───  INTERACTIVITY SETUP  ────────────────────────────────────────────────────
            //

            // ─── MOUSEDOWN ─────────────────────────────────────────────────────────────────

            let canvas_for_down = canvas.clone();
            let state_for_down = state_rc.clone();
            {
                let on_mousedown = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    // only left‐button
                    if event.button() == 0 {
                        let mut st = state_for_down.borrow_mut();
                        st.dragging = true;

                        let w = canvas_for_down.width() as f32;
                        let h = canvas_for_down.height() as f32;

                        // record starting mouse position (canvas‐relative)
                        let mx = (event.client_x() as f32) - w;
                        let my = (event.client_y() as f32) - h;
                        st.last_mouse_pos = (mx, my);

                        // prevent default so canvas doesn’t lose focus
                        event.prevent_default();
                    }
                }) as Box<dyn FnMut(_)>);

                canvas
                    .add_event_listener_with_callback(
                        "mousedown",
                        on_mousedown.as_ref().unchecked_ref()
                    )
                    .unwrap();
                on_mousedown.forget();
            }

            // ─── MOUSEMOVE ───

            {
                let canvas_for_move = canvas.clone();
                let state_for_move = state_rc.clone();

                let on_mousemove = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let mut st = state_for_move.borrow_mut();

                    if st.dragging {
                        let w = canvas_for_move.width() as f32;
                        let h = canvas_for_move.height() as f32;

                        // compute delta since last frame
                        let mx = (event.client_x() as f32) - w;
                        let my = (event.client_y() as f32) - h;

                        let (lx, ly) = st.last_mouse_pos;
                        let dx = mx - lx;
                        let dy = my - ly;
                        st.last_mouse_pos = (mx, my);

                        // update camera angles
                        st.camera.yaw -= dx * 0.005;
                        st.camera.pitch -= dy * 0.005;

                        // clamp pitch so we don’t flip upside‐down:
                        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01;
                        st.camera.pitch = st.camera.pitch.clamp(-max_pitch, max_pitch);
                    }
                }) as Box<dyn FnMut(_)>);

                canvas
                    .add_event_listener_with_callback(
                        "mousemove", on_mousemove.as_ref().unchecked_ref()
                    )
                    .unwrap();
                on_mousemove.forget();
            }

            // ─── MOUSEUP / MOUSELEAVE ───
            {
                let state_for_up = state_rc.clone();
                let on_mouseup = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
                    let mut st = state_for_up.borrow_mut();
                    st.dragging = false;
                }) as Box<dyn FnMut(_)>);

                canvas
                    .add_event_listener_with_callback(
                        "mouseup",
                        on_mouseup.as_ref().unchecked_ref()
                    )
                    .unwrap();
                canvas
                    .add_event_listener_with_callback(
                        "mouseleave",
                        on_mouseup.as_ref().unchecked_ref()
                    )
                    .unwrap(); // also stop dragging if pointer leaves
                on_mouseup.forget();
            }

            // ─── WHEEL (ZOOM) ───
            {
                let state_for_wheel = state_rc.clone();

                let on_wheel = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
                    let mut st = state_for_wheel.borrow_mut();
                    let delta = event.delta_y() as f32 * 0.01;
                    st.camera.distance = (st.camera.distance + delta).clamp(1.0, 50.0);
                    event.prevent_default();
                }) as Box<dyn FnMut(_)>);

                canvas
                    .add_event_listener_with_callback(
                        "wheel",
                        on_wheel.as_ref().unchecked_ref()
                    )
                    .unwrap();
                on_wheel.forget();
            }

            //
            // ───  END INTERACTIVITY SETUP ───────────────────────────────────────────────────
            //

            // ───  RENDER LOOP ────────────────────────────────────────────────────────────────

            // we’ll store the RAF callback so we can re‐schedule it each frame
            let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();
            let canvas_clone = canvas.clone();

            // now we kick off requestAnimationFrame(…) and draw each frame:
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
                // 1) borrow‐and‐render one frame
                {
                    let mut s = state_rc.borrow_mut();
                    s.render(&Some(canvas_clone.clone()));
                }

                // 2) schedule next frame
                web_sys::window()
                    .unwrap()
                    .request_animation_frame(
                        f.borrow()
                            .as_ref()
                            .unwrap()
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .unwrap();
            }) as Box<dyn FnMut(f64)>));

            // initial kick
            web_sys::window()
                .unwrap()
                .request_animation_frame(
                    g.borrow()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                )
                .unwrap();
        });
    });

    // 5) return the <canvas> in the view – Leptos mounts it, then our Effect hooks it.
    view! {
        <canvas
            id=canvas_id
            width=800
            height=600
            class="border w-full h-[500px]"
            style="border: 1px solid red;"
        ></canvas>
    }
}
