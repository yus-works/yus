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

use super::utils::{drag_head_to_cursor, make_spine_rpass, make_strip_rpass, solve_chain};

pub(crate) const CANVAS_ID: &str = "animals-canvas";

fn rotate_vec_static(v: Vec2, r: Vec2) -> Vec2 {
    Vec2::new(
        r.x * v.x - r.y * v.y, // x′ = cosφ·x − sinφ·y
        r.y * v.x + r.x * v.y, // y′ = sinφ·x + cosφ·y
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Joint {
    pub center: Vec2,
    pub axes: Vec2,

    dir: Vec2,
    angle: f32,  // radians
    dirty: bool, // if true, angle != atan2(dir.y, dir.x)
}

impl Joint {
    /// preferred: build from a direction vector (will be renormalised).
    pub fn new(center: Vec2, axes: Vec2, dir: Vec2) -> Self {
        let v = dir.normalize_or_zero();
        let theta = v.y.atan2(v.x);
        Self {
            center,
            axes,
            dir: v,
            angle: theta,
            dirty: false,
        }
    }

    /// convenience: build from an angle in radians.
    pub fn from_angle(center: Vec2, axes: Vec2, theta: f32) -> Self {
        Self {
            center,
            axes,
            dir: Vec2::new(theta.cos(), theta.sin()),
            angle: theta,
            dirty: false,
        }
    }

    /// Current direction (always unit-length).
    pub fn dir(&self) -> Vec2 {
        self.dir
    }

    /// Current angle in **radians**.
    pub fn angle(&mut self) -> f32 {
        if self.dirty {
            // recompute angle only if dir was changed
            self.angle = self.dir.y.atan2(self.dir.x);
            self.dirty = false;
        }
        self.angle
    }

    pub fn set_dir(&mut self, dir: Vec2) {
        self.dir = dir.normalize_or_zero();
        self.dirty = true; // cache needs refresh
    }

    pub fn set_angle(&mut self, theta: f32) {
        self.angle = theta;
        self.dir = Vec2::new(theta.cos(), theta.sin());
        self.dirty = false;
    }

    fn normal_points(&self, v: Vec2) -> Option<(Vec2, Vec2)> {
        if v.length_squared() == 0.0 {
            return None;
        }

        let n1 = Vec2::new(-v.y,  v.x);
        let n2 = Vec2::new( v.y, -v.x);

        let dir_inv = Vec2::new(self.dir.x, -self.dir.y);   // (cosθ, −sinθ)

        let hit = |d: Vec2| -> Vec2 {
            let d_local = rotate_vec_static(d, dir_inv);

            let denom =
                (d_local.x * d_local.x) / (self.axes.x * self.axes.x) +
                (d_local.y * d_local.y) / (self.axes.y * self.axes.y);

            let s = 1.0 / denom.sqrt();

            self.rotate_vec(d_local * s) + self.center
        };

        Some((hit(n1), hit(n2)))
    }

    #[inline]
    pub fn rotate_vec(&self, v: Vec2) -> Vec2 {
        rotate_vec_static(v, self.dir)
    }

    pub fn rotate_self(&mut self, r: Vec2) {
        let r = r.normalize_or_zero();

        self.center = rotate_vec_static(self.center, r);
        self.dir = rotate_vec_static(self.dir, r);
        // still unit-length because both inputs were
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
                if i == joints.len() - 1 {
                    // flip tangent vector that initially looks behind
                    // so it looks forward like every other
                    -(j.center - joints[i - 1].center).normalize()
                } else {
                    (j.center - joints[i + 1].center).normalize()
                }
            };

            if let Some((n1, n2)) = j.normal_points(v) {
                skin.push_front(n1);
                skin.push_back(n2);
            }
        }

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
        Joint::new(Vec2::ZERO, Vec2::new(0.1, 0.10), Vec2::ZERO),
        Joint::new(Vec2::ZERO, Vec2::new(0.1, 0.15), Vec2::ZERO),
        Joint::new(Vec2::ZERO, Vec2::new(0.1, 0.20), Vec2::ZERO),
        Joint::new(Vec2::ZERO, Vec2::new(0.1, 0.25), Vec2::ZERO),
    ];

    let snake = {
        let pts = points_rc.borrow();

        let mut i = 0;
        for (p, j) in pts.iter().zip(&mut joints) {
            j.center = *p;

            let dir = {
                if i == pts.len() - 1 {
                    -(p - pts[i - 1]).normalize()
                } else {
                    (p - pts[i + 1]).normalize()
                }
            };

            j.set_dir(dir);

            i += 1;
        }

        Animal::new(joints.clone())
    };

    let joints_rc = Rc::new(RefCell::new(joints));

    let (strip_pass, strip_pipe) = make_strip_rpass(
        Rc::new(RefCell::new(snake.skin.clone())),
        0.05,
        vs_src,
        fs_src,
    );
    let (spine_pass, spine_pipe) = make_spine_rpass(joints_rc.clone(), vs_src, fs_src);

    {
        let vs_src = vs_src.clone();
        let fs_src = fs_src.clone();
        let pipes = [strip_pipe.clone(), spine_pipe.clone()];

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
            strip_pass,
            spine_pass,
            make_points_rpass(Rc::new(RefCell::new(snake.skin.clone())), [0., 1., 0., 0.]),
            make_points_rpass(points_rc.clone(), [1., 0., 0., 0.]),
        ],
        drag_head_to_cursor(points_rc.clone()),
        solve_chain(points_rc.clone(), 0.15, 9),
    );

    view! {
        { super::view::canvas(gpu_support, show_hint) }
    }
}
