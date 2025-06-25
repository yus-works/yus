use std::{cell::RefCell, rc::Rc};

use glam::{Mat4, Vec2, Vec3};
use leptos::prelude::{GetUntracked, RwSignal};
use web_sys::{HtmlCanvasElement, PointerEvent};

use crate::{
    components::{
        demo::to_clip_space,
        demos::utils::{add_listener, InstanceCtx, RenderPass},
    },
    meshes::{
        quad::{QUAD_INDICES, QUAD_VERTS},
        utils::stroke_polyline,
    },
    render::renderer::{
        camera_input::CameraInput,
        gpu::{
            gpu_state::{
                create_idx_buff_init, create_instance_buff, create_vert_buff_init, ensure_instance_capacity, FrameCtx, Projection
            }, GpuState
        },
        instance::InstanceRaw,
        vertex::Vertex,
    },
};

fn make_joint_pipe(st: &GpuState, vs_src: &str, fs_src: &str) -> wgpu::RenderPipeline {
    let color_target = Some(wgpu::ColorTargetState {
        format: st.surface_context.config.format, // same as the swap-chain
        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
    });

    let vs = st
        .surface_context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vs shader with custom topology"),
            source: wgpu::ShaderSource::Wgsl(vs_src.into()),
        });

    let fs = st
        .surface_context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fs shader with custom topology"),
            source: wgpu::ShaderSource::Wgsl(fs_src.into()),
        });

    let layout =
        st.surface_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("spine layout"),
                bind_group_layouts: &[&st.resource_context.common_bind_group.layout],
                push_constant_ranges: &[],
            });

    st.surface_context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("joints pipe"),
            layout: Some(&layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("joints_vs"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("joints_fs"),
                targets: &[color_target],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        })
}

/// Build one model matrix per joint.
fn build_instance_mats(points: &[Vec2], big: f32) -> Vec<InstanceRaw> {
    points
        .iter()
        .enumerate()
        .map(|(_, p)| {
            let r = big;
            let model = Mat4::from_scale_rotation_translation(
                Vec3::new(r, r, 1.0),    // xy-scale, no z-thickness
                glam::Quat::IDENTITY,    // no rotation
                Vec3::new(p.x, p.y, 0.), // clip-space translation
            );
            InstanceRaw::from_mat4(model)
        })
        .collect()
}

pub(crate) fn make_spine_rpass(
    points: Rc<RefCell<Vec<Vec2>>>,
    vs_src: RwSignal<String>,
    fs_src: RwSignal<String>,
) -> (RenderPass, Rc<RefCell<Option<wgpu::RenderPipeline>>>) {
    let pipeline = Rc::new(RefCell::new(None));

    // clone handles that must live inside the closure
    let pipe_handle = pipeline.clone();
    let pts_handle = points.clone();
    let vs_handle = vs_src.clone();
    let fs_handle = fs_src.clone();

    let vbuf_handle: Rc<RefCell<Option<wgpu::Buffer>>> = Rc::new(RefCell::new(None));
    let ibuf_handle: Rc<RefCell<Option<wgpu::Buffer>>> = Rc::new(RefCell::new(None));
    let inst_handle: Rc<RefCell<Option<InstanceCtx>>> = Rc::new(RefCell::new(None));

    let pass = Rc::new(RefCell::new(
        move |st: &mut GpuState, _cam: &CameraInput, ctx: &mut FrameCtx| {
            if pipe_handle.borrow().is_none() {
                *pipe_handle.borrow_mut() = Some(make_joint_pipe(
                    st,
                    &vs_handle.get_untracked(),
                    &fs_handle.get_untracked(),
                ));
            }

            if vbuf_handle.borrow().is_none() {
                *vbuf_handle.borrow_mut() = Some(create_vert_buff_init(
                    &st.surface_context,
                    QUAD_VERTS,
                ));
                *ibuf_handle.borrow_mut() = Some(create_idx_buff_init(
                    &st.surface_context,
                    QUAD_INDICES,
                ));
            }

            if inst_handle.borrow().is_none() {
                let instances = build_instance_mats(&pts_handle.borrow(), 0.05);
                let needed = instances.len() as u32;
                let count = instances.len() as u32;
                let buff = create_instance_buff(&st.surface_context, 256);

                *inst_handle.borrow_mut() = Some(InstanceCtx {
                    instances,
                    needed,
                    count,
                    buff
                });
            }

            let binding = inst_handle.borrow();
            let inst = binding.as_ref().unwrap();

            ensure_instance_capacity(st, inst_handle.borrow().as_ref().unwrap().needed);
            st.surface_context.queue.write_buffer(
                &inst.buff,
                0,
                bytemuck::cast_slice(&inst.instances),
            );

            // 3) bind + draw
            let mut rp = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("circle pass"),
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

            let binding = vbuf_handle.borrow();
            let vbuf = binding.as_ref().unwrap();
            rp.set_vertex_buffer(0, vbuf.slice(..));

            let binding = inst_handle.borrow();
            let inst = binding.as_ref().unwrap();
            rp.set_vertex_buffer(1, inst.buff.slice(..));

            let binding = ibuf_handle.borrow();
            let ibuf = binding.as_ref().unwrap();
            rp.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint16);

            rp.set_bind_group(0, &st.resource_context.common_bind_group.group, &[]);

            let idx_count = QUAD_INDICES.len() as u32;
            let inst_count = inst_handle.borrow().as_ref().unwrap().count;
            rp.draw_indexed(0..idx_count, 0, 0..inst_count);
        },
    ));

    (pass, pipeline)
}

fn make_bones_pipe(st: &GpuState, vs_src: &str, fs_src: &str) -> wgpu::RenderPipeline {
    let vs = st
        .surface_context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("animals vs"),
            source: wgpu::ShaderSource::Wgsl(vs_src.into()),
        });

    let fs = st
        .surface_context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("animals fs"),
            source: wgpu::ShaderSource::Wgsl(fs_src.into()),
        });

    let layout =
        st.surface_context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bones layout"),
                bind_group_layouts: &[&st.resource_context.common_bind_group.layout],
                push_constant_ranges: &[],
            });

    st.surface_context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bones pipe"),
            layout: Some(&layout),
            cache: None,
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("bones_vs"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("bones_fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: st.surface_context.config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        })
}

pub(crate) fn make_strip_rpass(
    points: Rc<RefCell<Vec<Vec2>>>,
    vs_src: RwSignal<String>,
    fs_src: RwSignal<String>,
) -> (RenderPass, Rc<RefCell<Option<wgpu::RenderPipeline>>>) {
    let pipeline = Rc::new(RefCell::new(None));

    // clone handles that must live inside the closure
    let pipe_handle = pipeline.clone();
    let pts_handle = points.clone();
    let vs_handle = vs_src.clone();
    let fs_handle = fs_src.clone();

    let vbuf_handle:  Rc<RefCell<Option<wgpu::Buffer>>> = Rc::new(RefCell::new(None));
    let mut v_count = 0;

    let pass = Rc::new(RefCell::new(
        move |st: &mut GpuState, cam: &CameraInput, ctx: &mut FrameCtx| {
            st.populate_common_buffers(&Projection::FlatQuad, cam);

            let vs = vs_handle.get_untracked();
            let fs = fs_handle.get_untracked();

            // (re)build if missing
            if pipe_handle.borrow().is_none() {
                *pipe_handle.borrow_mut() = Some(make_bones_pipe(&st, &vs, &fs));
            }

            if vbuf_handle.borrow().is_none() {
                let verts = {
                    let pts = pts_handle.borrow();
                    stroke_polyline(&pts, 0.15)
                };

                // NOTE: no indices needed because this is a strip
                st.num_indices = 0;

                *vbuf_handle.borrow_mut() = Some(create_vert_buff_init(
                    &st.surface_context,
                    &verts,
                ));

                v_count = verts.len();
            }

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

            let binding = vbuf_handle.borrow();
            let vbuf = binding.as_ref().unwrap();

            rp.set_vertex_buffer(0, vbuf.slice(..));
            rp.set_bind_group(0, &st.resource_context.common_bind_group.group, &[]);

            rp.draw(0..v_count as u32, 0..1);
        },
    ));

    (pass, pipeline)
}

pub(crate) fn click_add_points(
    points_rc: Rc<RefCell<Vec<Vec2>>>,
) -> impl FnOnce(&HtmlCanvasElement) {
    let pts_master = points_rc.clone(); // one shared handle

    move |canvas: &HtmlCanvasElement| {
        // one handle just for registering the listener (borrowed immutably)
        let canvas_ref = canvas.clone();

        // a second handle that the callback owns outright
        let canvas_owned = canvas.clone();

        let pts = pts_master.clone();

        add_listener::<web_sys::PointerEvent, _>(
            &canvas_ref, // immutable borrow lives only for this call
            "pointerdown",
            move |e| {
                if e.button() != 0 {
                    return;
                }

                // use the owned handle inside
                let p = to_clip_space(&e, &canvas_owned);
                pts.borrow_mut().push(p);
            },
        );
    }
}

pub(crate) fn solve_chain(
    joints_rc: Rc<RefCell<Vec<Vec2>>>,
    seg_len: f32,
    iterations: usize,
) -> impl FnMut() + Clone {
    move || {
        let mut pts = joints_rc.borrow_mut();
        for _ in 0..iterations {
            for i in 1..pts.len() {
                let dir = pts[i] - pts[i - 1];
                let dist = dir.length();
                if dist != 0.0 {
                    pts[i] = pts[i - 1] + dir * (seg_len / dist);
                }
            }
        }
    }
}

pub(crate) fn drag_head_to_cursor(
    points_rc: Rc<RefCell<Vec<Vec2>>>,
) -> impl Fn(&HtmlCanvasElement) + Clone {
    // `dragging` flag must be shareable by both inner callbacks
    let dragging = Rc::new(RefCell::new(false));

    move |canvas: &HtmlCanvasElement| {
        let pts_move = points_rc.clone();
        let pts_move2 = points_rc.clone();
        let dragging_pd = dragging.clone();
        let dragging_mv = dragging.clone();

        // pointer down
        {
            // canvas_ref only borrowed for add_event_listener
            let canvas_ref = canvas.clone();
            let canvas_for_math = canvas.clone(); // moved into closure

            add_listener::<PointerEvent, _>(&canvas_ref, "pointerdown", move |e| {
                if e.button() != 0 {
                    return;
                }
                *dragging_pd.borrow_mut() = true;

                // initial snap
                let p = to_clip_space(&e, &canvas_for_math);
                if let Some(first) = pts_move.borrow_mut().first_mut() {
                    *first = p;
                }
                e.prevent_default();
            });
        }

        // pointer move : update head while dragging
        {
            let canvas_ref = canvas.clone(); // for registering
            let canvas_for_math = canvas.clone(); // moved in
            add_listener::<PointerEvent, _>(&canvas_ref, "pointermove", move |e| {
                if !*dragging_mv.borrow() {
                    return;
                }
                let p = to_clip_space(&e, &canvas_for_math);
                if let Some(first) = pts_move2.borrow_mut().first_mut() {
                    *first = p;
                }
                e.prevent_default();
            });
        }

        // pointer up
        {
            let canvas_ref = canvas.clone();
            let dragging_up = dragging.clone();
            add_listener::<PointerEvent, _>(&canvas_ref, "pointerup", move |_e| {
                *dragging_up.borrow_mut() = false;
            });
            let canvas_ref = canvas.clone();
            let dragging_leave = dragging.clone();
            add_listener::<PointerEvent, _>(&canvas_ref, "pointerleave", move |_e| {
                *dragging_leave.borrow_mut() = false;
            });
        }
    }
}
