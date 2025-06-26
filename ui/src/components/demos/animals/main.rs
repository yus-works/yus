use leptos::{
    component, prelude::{Effect, Get, RwSignal}, view, IntoView
};

use glam::Vec2;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{
    components::demos::utils::{make_points_rpass, start_rendering},
    meshes,
    render::renderer::{camera_input::CameraInput, gpu::GpuState},
};

use super::utils::{drag_head_to_cursor, make_strip_rpass, make_spine_rpass, solve_chain};

pub(crate) const CANVAS_ID: &str = "animals-canvas";

#[derive(Clone)]
pub struct Joint {
    pub center: Vec2,
    pub axes: Vec2,
    angle: f32,
}

impl Joint {
    fn ellipse_point(&self, t: f32) -> Vec2 {
        let a = self.axes.x;
        let b = self.axes.y;
        let cos_t = t.cos();
        let sin_t = t.sin();
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();
        let c_x = self.center.x;
        let c_y = self.center.y;

        Vec2::new(
            c_x + a * cos_t * cos_a - b * sin_t * sin_a,
            c_y + a * cos_t * sin_a + b * sin_t * cos_a,
        )
    }

    fn normal_points(&self, v: Vec2) -> Option<(Vec2, Vec2)> {
        if v.length_squared() == 0.0 { return None; }

        let n1 = Vec2::new(-v.y,  v.x);
        let n2 = Vec2::new( v.y, -v.x);

        let hit = |d: Vec2| -> Vec2 {
            let d_local = Joint::rotate_vec(d, -self.angle);

            let denom = (d_local.x*d_local.x) /
                (self.axes.x*self.axes.x) +
                (d_local.y*d_local.y) /
                (self.axes.y*self.axes.y);

            let s = 1.0 / denom.sqrt();

            // local -> world
            Joint::rotate_vec(d_local * s, self.angle) + self.center
        };

        let p1 = hit(n1);
        let p2 = hit(n2);
        Some((p1, p2))
    }

    fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(c * v.x - s * v.y, s * v.x + c * v.y)
    }
}

struct Animal {
    spine: Vec<Joint>,
    skin: Vec<Vec2>,
}

impl Animal {
    fn new(joints: Vec<Joint>) -> Animal {
        let mut skin = VecDeque::new();
        for (i, j) in joints.iter().enumerate() {
            let v = {
                if i == joints.len() - 1 { (j.center - joints[i - 1].center).normalize() }
                else { (j.center - joints[i + 1].center).normalize() }
            };

            if let Some((n1, n2)) = j.normal_points(v) {
                skin.push_front(n1);
                skin.push_back(n2);
            }
        };

        Animal {
            spine: joints,
            skin: skin.into(),
        }
    }
}

#[component]
pub fn Animals(vs_src: RwSignal<String>, fs_src: RwSignal<String>) -> impl IntoView {
    let state_rc: Rc<RefCell<Option<GpuState>>> = Rc::new(RefCell::new(None));

    let points_rc: Rc<RefCell<Vec<Vec2>>> = Rc::new(RefCell::new(meshes::strip::worm()));

    let camera_rc: Rc<RefCell<Option<CameraInput>>> = Rc::new(RefCell::new(None));

    let gpu_support = RwSignal::new(true);
    let show_hint = RwSignal::new(true);

    let mut joints = vec![
        Joint { center: Vec2::ZERO, axes: Vec2::new(0.3, 0.1), angle: 0. },
        Joint { center: Vec2::ZERO, axes: Vec2::new(0.3, 0.1), angle: 0. },
    ];

    let snake = {
        let pts = points_rc.borrow();

        for (p, j) in pts.iter().zip(&mut joints) {
            j.center = *p;
        }

        Animal::new(joints.clone())
    };

    let joints_rc = Rc::new(RefCell::new(joints));

    let (strip_pass, strip_pipe) = make_strip_rpass(Rc::new(RefCell::new(snake.skin.clone())), 0.05, vs_src, fs_src);
    let (spine_pass, spine_pipe) = make_spine_rpass(joints_rc.clone(), vs_src, fs_src);

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
        CANVAS_ID,
        vec![
            strip_pass, spine_pass,
            make_points_rpass(Rc::new(RefCell::new(snake.skin.clone())), [0., 1., 0., 0.]),
            make_points_rpass(points_rc.clone(), [1., 0., 0., 0.])
        ],
        drag_head_to_cursor(points_rc.clone()),
        solve_chain(points_rc.clone(), 0.15, 9),
    );

    view! {
        { super::view::canvas(gpu_support, show_hint) }
    }
}
