use leptos::{
    component, leptos_dom::logging::console_log, prelude::{Effect, Get, RwSignal, Set}, view, IntoView
};

use glam::Vec2;
use std::{cell::RefCell, rc::Rc};

use crate::{
    components::demos::utils::start_rendering,
    meshes,
    render::renderer::{camera_input::CameraInput, gpu::GpuState},
};

use super::utils::{drag_head_to_cursor, make_strip_rpass, make_spine_rpass, solve_chain};

pub(crate) const CANVAS_ID: &str = "animals-canvas";

#[component]
pub fn Animals(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));
    let pending = RwSignal::new(None::<(String, String)>);

    let points_rc: Rc<RefCell<Vec<Vec2>>> = Rc::new(RefCell::new(meshes::strip::worm()));

    let camera_rc: Rc<RefCell<Option<CameraInput>>> = Rc::new(RefCell::new(None));

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    let (strip_pass, strip_pipe) = make_strip_rpass(points_rc.clone(), vs_src, fs_src);
    let (spine_pass, spine_pipe) = make_spine_rpass(points_rc.clone(), vs_src, fs_src);

    {
        let vs_src = vs_src.clone();
        let fs_src = fs_src.clone();
        let pipes = [
            strip_pipe.clone(),
            spine_pipe.clone()
        ];

        Effect::new(move |_| {
            vs_src.get();
            fs_src.get();
            for p in &pipes {
                *p.borrow_mut() = None;
            }
        });
    }

    start_rendering(
        state_rc,
        camera_rc,
        show_hint,
        gpu_support,
        pending,
        CANVAS_ID,
        vec![strip_pass, spine_pass],
        vec![strip_pipe, spine_pipe],
        drag_head_to_cursor(points_rc.clone()),
        solve_chain(points_rc.clone(), 0.15, 9),
    );

    view! {
        { super::view::canvas(gpu_support, show_hint) }
    }
}
