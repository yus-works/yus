use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::Effect;
use leptos::prelude::StyleAttribute;
use leptos::view;
use leptos::prelude::ClassAttribute;
use leptos::prelude::GlobalAttributes;

use leptos::component;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
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
            // wait until the <canvas> actually exists
            TimeoutFuture::new(0).await;

            // grab the DOM canvas
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas: HtmlCanvasElement = document
                .get_element_by_id(&id)
                .expect("canvas not in DOM yet")
                .dyn_into::<HtmlCanvasElement>()
                .expect("element is not a canvas");

            // init WGPU with that canvas
            let state = match init_wgpu(&canvas).await {
                Ok(s) => s,
                Err(err) => {
                    web_sys::console::error_1(&format!("WGPU init failed: {:?}", err).into());
                    return;
                }
            };

            // ???
            let state_rc: Rc<RefCell<_>> = Rc::new(RefCell::new(state));

            // ?????
            let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();
            let canvas_clone = canvas.clone();

            *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_: f64| {
                // 1) borrow-and-render one frame, passing `&Some(canvas.clone())`:
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

            // kick off loop
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
