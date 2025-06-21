use glam::Vec2;
use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GetUntracked, GlobalAttributes, RwSignal, Set, Show
};
use web_sys::{HtmlCanvasElement, PointerEvent};
use std::cell::RefCell;
use std::rc::Rc;

use leptos::view;

use super::utils;
use super::utils::RenderPass;
use crate::components::demo::{make_pipe_with_topology_and_layout, to_clip_space};
use crate::meshes::utils::stroke_polyline;
use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::GpuState;
use crate::render::renderer::gpu::gpu_state::{FrameCtx, Projection};
use crate::render::renderer::gpu::gpu_state::create_vert_buff;
use crate::render::renderer::mesh;
use leptos::IntoView;
use leptos::component;

pub fn make_strip_rpass(
    points : Rc<RefCell<Vec<Vec2>>>,
    vs_src : RwSignal<String>,
    fs_src : RwSignal<String>,
) -> (RenderPass, Rc<RefCell<Option<wgpu::RenderPipeline>>>) {
    let pipeline = Rc::new(RefCell::new(None));

    // clone handles that must live inside the closure
    let pipe_handle  = pipeline.clone();
    let pts_handle   = points.clone();
    let vs_handle    = vs_src.clone();
    let fs_handle    = fs_src.clone();

    let pass = Rc::new(RefCell::new(
        move |st: &mut GpuState, cam: &CameraInput, ctx: &mut FrameCtx| {

            st.populate_common_buffers(
                &Projection::FlatQuad,
                cam,
                &mesh::CpuMesh::new(vec![], vec![])
            );

            let vs = vs_handle.get_untracked();
            let fs = fs_handle.get_untracked();

            // (re)build if missing
            if pipe_handle.borrow().is_none() {
                *pipe_handle.borrow_mut() = Some(make_pipe_with_topology_and_layout(
                    &st.surface_context.device,
                    st.surface_context.config.format,
                    wgpu::PrimitiveTopology::TriangleStrip,
                    &[
                        &st.resource_context.common_bind_group.layout.clone(),
                    ],
                    &vs,
                    &fs,
                ));
            }

            let verts = {
                let pts = pts_handle.borrow();
                stroke_polyline(&pts, 0.05)
            };
            st.vertex_buffer = create_vert_buff(&st.surface_context, &verts);
            st.num_indices   = 0;

            let mut rp = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("polyline pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &ctx.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            rp.set_pipeline(pipe_handle.borrow().as_ref().unwrap());
            rp.set_vertex_buffer(0, st.vertex_buffer.slice(..));
            rp.set_bind_group(0, &st.resource_context.common_bind_group.group, &[]);
            rp.draw(0..verts.len() as u32, 0..1);
        }
    ));

    (pass, pipeline)
}


#[component]
pub fn Animals(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let canvas_id = "animals-demo-canvas";

    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));
    let pending = RwSignal::new(None::<(String, String)>);

    let points_rc: Rc<RefCell<Vec<Vec2>>> = Rc::new(RefCell::new(vec![
        Vec2::new(-0.8, -0.7),
        Vec2::new(-0.7, -0.6),
        Vec2::new(-0.6, -0.5),
        Vec2::new(-0.5, -0.4),
        Vec2::new(-0.4, -0.3),
        Vec2::new(-0.3, -0.2),
        Vec2::new(-0.2, -0.1),
        Vec2::new(-0.1,  0.0),
        Vec2::new( 0.0,  0.1),
        Vec2::new( 0.1,  0.2),
        Vec2::new( 0.2,  0.3),
        Vec2::new( 0.3,  0.4),
        Vec2::new( 0.4,  0.5),
        Vec2::new( 0.5,  0.6),
        Vec2::new( 0.6,  0.7),
        Vec2::new( 0.7,  0.8),
        Vec2::new( 0.8,  0.9),
        Vec2::new( 0.9,  1.0),
    ]));

    let camera_rc: Rc<RefCell<Option<CameraInput>>> = Rc::new(RefCell::new(None));

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    {
        let pending = pending.clone();
        Effect::new(move |_| {
            pending.set(Some((vs_src.get(), fs_src.get())));
        });
    }

    let (strip_pass, strip_pipe) = make_strip_rpass(points_rc.clone(), vs_src, fs_src);

    utils::start_rendering(
        state_rc,
        camera_rc,
        show_hint,
        gpu_support,
        pending,
        canvas_id,

        vec![strip_pass],
        vec![strip_pipe],

        drag_tail_to_cursor(points_rc.clone()),
        solve_chain(points_rc.clone(), 0.05, 9),
    );

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

fn click_add_points(points_rc: Rc<RefCell<Vec<Vec2>>>) -> impl FnOnce(&HtmlCanvasElement) {
    let pts_master = points_rc.clone();          // one shared handle

    move |canvas: &HtmlCanvasElement| {
        // one handle just for registering the listener (borrowed immutably)
        let canvas_ref = canvas.clone();

        // a second handle that the callback owns outright
        let canvas_owned = canvas.clone();

        let pts = pts_master.clone();

        utils::add_listener::<web_sys::PointerEvent, _>(
            &canvas_ref,                   // immutable borrow lives only for this call
            "pointerdown",
            move |e| {
                if e.button() != 0 { return; }

                // use the owned handle inside
                let p = to_clip_space(&e, &canvas_owned);
                pts.borrow_mut().push(p);
            },
        );
    }
}

fn solve_chain(
    joints_rc : Rc<RefCell<Vec<Vec2>>>,
    seg_len   : f32,
    iterations: usize,
) -> impl FnMut() + Clone {
    move || {
        let mut pts = joints_rc.borrow_mut();
        for _ in 0..iterations {
            for i in 1..pts.len() {
                let dir  = pts[i] - pts[i - 1];
                let dist = dir.length();
                if dist != 0.0 {
                    pts[i] = pts[i - 1] + dir * (seg_len / dist);
                }
            }
        }
    }
}

pub fn drag_tail_to_cursor(
    points_rc: Rc<RefCell<Vec<Vec2>>>,
) -> impl Fn(&HtmlCanvasElement) + Clone {
    // `dragging` flag must be shareable by both inner callbacks
    let dragging = Rc::new(RefCell::new(false));

    // ── The outer closure ───────────────────────────────────────────────
    // *no captures are moved*, only cloned -> implements Fn + Clone
    move |canvas: &HtmlCanvasElement| {
        // Quickly clone handles that will be moved into each listener
        let pts_move      = points_rc.clone();
        let pts_move2     = points_rc.clone();
        let dragging_pd   = dragging.clone();
        let dragging_mv   = dragging.clone();

        // ----- POINTER-DOWN : start dragging & snap tail -----------------
        {
            // canvas_ref only borrowed for add_event_listener
            let canvas_ref = canvas.clone();
            let canvas_for_math = canvas.clone();     // moved into closure

            utils::add_listener::<PointerEvent, _>(
                &canvas_ref,
                "pointerdown",
                move |e| {
                    if e.button() != 0 { return; }
                    *dragging_pd.borrow_mut() = true;

                    // initial snap
                    let p = to_clip_space(&e, &canvas_for_math);
                    if let Some(first) = pts_move.borrow_mut().first_mut() {
                        *first = p;
                    }
                    e.prevent_default();
                },
            );
        }

        // pointer move : update tail while dragging
        {
            let canvas_ref = canvas.clone(); // for registering
            let canvas_for_math = canvas.clone(); // moved in
            utils::add_listener::<PointerEvent, _>(
                &canvas_ref,
                "pointermove",
                move |e| {
                    if !*dragging_mv.borrow() { return; }
                    let p = to_clip_space(&e, &canvas_for_math);
                    if let Some(first) = pts_move2.borrow_mut().first_mut() {
                        *first = p;
                    }
                    e.prevent_default();
                },
            );
        }

        // pointer up
        {
            let canvas_ref = canvas.clone();
            let dragging_up = dragging.clone();
            utils::add_listener::<PointerEvent, _>(
                &canvas_ref,
                "pointerup",
                move |_e| { *dragging_up.borrow_mut() = false; },
            );
            let canvas_ref = canvas.clone();
            let dragging_leave = dragging.clone();
            utils::add_listener::<PointerEvent, _>(
                &canvas_ref,
                "pointerleave",
                move |_e| { *dragging_leave.borrow_mut() = false; },
            );
        }
    }
}
