use glam::Vec2;

use crate::render::renderer::vertex::Vertex;

use super::utils::stroke_polyline;

pub fn strip() -> Vec<Vertex> {
    stroke_polyline(&[
        Vec2::new(-1.0, -1.0), 
        Vec2::new(-0.8, -0.8), 
        Vec2::new(-0.6, -0.8), 
        Vec2::new(-0.4, -0.8), 
        Vec2::new(-0.2, -0.8), 
        Vec2::new(-0.2, -0.6), 
        Vec2::new(-0.2, -0.4), 
        Vec2::new(-0.2, -0.2), 
        Vec2::new(0.0, 0.0), 
    ], 0.05)
}
