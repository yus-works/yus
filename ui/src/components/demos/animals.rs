use glam::Vec2;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::{
    ClassAttribute, Effect, ElementChild, Get, GlobalAttributes, RwSignal, Set, Show,
};
use web_sys::HtmlCanvasElement;
use std::cell::RefCell;
use std::rc::Rc;

use leptos::view;

use super::utils;
use super::utils::RenderPass;
use crate::components::demo::{make_pipeline_with_topology, to_clip_space};
use crate::meshes;
use crate::meshes::utils::stroke_polyline;
use crate::render::renderer::camera_input::CameraInput;
use crate::render::renderer::gpu::GpuState;
use crate::render::renderer::gpu::gpu_state::FrameCtx;
use crate::render::renderer::gpu::gpu_state::Projection;
use crate::render::renderer::gpu::gpu_state::create_idx_buff;
use crate::render::renderer::gpu::gpu_state::create_vert_buff;
use crate::render::renderer::mesh::CpuMesh;
use leptos::IntoView;
use leptos::component;

use crate::render::renderer::vertex::Vertex;

/// two little quads (triangle-list) that sit flush against either end
pub const END_VERTS: &[Vertex] = &[
    // -- left quad (-1.0 … -0.8) --
    Vertex {
        position: [-1.00, -0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-0.80, -0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.80, 0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-1.00, 0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [0.0, 1.0],
    },
    // -- right quad ( +0.8 … +1.0 ) --
    Vertex {
        position: [0.80, -0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.00, -0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.00, 0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.80, 0.25, 0.0],
        normal: [0., 0., 1.],
        uv: [0.0, 1.0],
    },
];

pub const END_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // left
    4, 5, 6, 4, 6, 7, // right
];

pub fn make_strip_rpass(points: Rc<RefCell<Vec<Vec2>>>) -> RenderPass {
    let pipeline: Rc<RefCell<Option<wgpu::RenderPipeline>>> = Rc::new(RefCell::new(None));

    Rc::new(RefCell::new(
        move |st: &mut GpuState, _cam: &CameraInput, ctx: &mut FrameCtx| {
            // 1. lazy-build pipeline ---------------------------------------------------------
            if pipeline.borrow().is_none() {
                let p = make_pipeline_with_topology(
                    &st.surface_context.device,
                    st.surface_context.config.format,
                    wgpu::PrimitiveTopology::TriangleStrip,
                    include_str!("../../render/renderer/shaders/fish.vert.wgsl"),
                    include_str!("../../render/renderer/shaders/fish.frag.wgsl"),
                );
                *pipeline.borrow_mut() = Some(p);
            }

            // 2. build verts from current points --------------------------------------------
            let verts = {
                let pts = points.borrow();
                stroke_polyline(&pts, 0.05)
            };
            st.vertex_buffer = create_vert_buff(&st.surface_context, &verts);
            st.num_indices   = 0;

            // 3. draw -----------------------------------------------------------------------
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

            rp.set_pipeline(pipeline.borrow().as_ref().unwrap());
            rp.set_vertex_buffer(0, st.vertex_buffer.slice(..));
            rp.draw(0..verts.len() as u32, 0..1);
        }
    ))
}

pub fn make_end_quads_rpass() -> RenderPass {
    // verts + indices baked
    let mesh = CpuMesh::new(END_VERTS.to_vec(), END_INDICES.to_vec());
    let proj = Rc::new(RefCell::new(Projection::FlatQuad));

    let pipeline: Rc<RefCell<Option<wgpu::RenderPipeline>>> = Rc::new(RefCell::new(None));
    let vs_src = include_str!("../../render/renderer/shaders/fish.vert.wgsl");
    let fs_src = include_str!("../../render/renderer/shaders/fish.frag.wgsl");

    Rc::new(RefCell::new(
        move |st: &mut GpuState, _cam: &CameraInput, ctx: &mut FrameCtx| {
            if pipeline.borrow().is_none() {
                let p = make_pipeline_with_topology(
                    &st.surface_context.device,
                    st.surface_context.config.format,
                    wgpu::PrimitiveTopology::TriangleList,
                    &vs_src,
                    &fs_src,
                );
                *pipeline.borrow_mut() = Some(p);
            }

            st.vertex_buffer = create_vert_buff(&st.surface_context, mesh.vertices.as_slice());
            st.index_buffer = create_idx_buff(&st.surface_context, mesh.indices.as_slice());
            st.num_indices = mesh.index_count;

            let mut rp = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("end-quads pass"),
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

            rp.set_pipeline(pipeline.borrow().as_ref().unwrap());
            rp.set_vertex_buffer(0, st.vertex_buffer.slice(..));
            rp.set_index_buffer(st.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rp.draw_indexed(0..st.num_indices, 0, 0..1);
        },
    ))
}

#[component]
pub fn Animals(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let canvas_id = "animals-demo-canvas";

    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));
    let pending = RwSignal::new(None::<(String, String)>);

    let points_rc: Rc<RefCell<Vec<Vec2>>> = Rc::new(RefCell::new(vec![
        Vec2::new(-1.0, -1.0),
        Vec2::new(-0.8, -0.8),
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

    utils::start_rendering(
        state_rc,
        camera_rc,
        show_hint,
        gpu_support,
        pending,
        canvas_id,
        vec![make_strip_rpass(points_rc.clone()), make_end_quads_rpass()],

           {
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
            },

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
