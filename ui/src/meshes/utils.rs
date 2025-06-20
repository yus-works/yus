use glam::Vec2;
use crate::render::renderer::vertex::Vertex;

/// Extrude a 2-D poly-line into a triangle-strip line of constant thickness.
///
/// `points`   – poly-line in (−1..1)
/// `width`    – total line width (same space)  
pub fn stroke_polyline(points: &[Vec2], width: f32) -> Vec<Vertex> {
    assert!(points.len() >= 2, "need at least two points");
    let half_w = width * 0.5;
    let mut out = Vec::with_capacity(points.len() * 2);

    // Pre-compute cumulative distances for UVs
    let mut dist = Vec::with_capacity(points.len());
    dist.push(0.0);
    for i in 1..points.len() {
        let d = dist[i - 1] + points[i].distance(points[i - 1]);
        dist.push(d);
    }
    let total_len = *dist.last().unwrap_or(&1.0);

    for (i, &p) in points.iter().enumerate() {
        // 1. tangent
        let t = if i == 0 {
            (points[1] - p).normalize()
        } else if i == points.len() - 1 {
            (p - points[i - 1]).normalize()
        } else {
            (points[i + 1] - points[i - 1]).normalize()
        };

        // 2. normal (screen-space left)
        let n = Vec2::new(-t.y, t.x);

        // 3. two extruded verts
        let left  = p + n * half_w;
        let right = p - n * half_w;

        // 4. running-length-based U coord
        let u = dist[i] / total_len;

        // Push L then R
        out.push(Vertex {
            position: [left.x,  left.y, 0.0],
            normal:   [0.0, 0.0, 1.0],
            uv:       [u, 0.0],
        });
        out.push(Vertex {
            position: [right.x, right.y, 0.0],
            normal:   [0.0, 0.0, 1.0],
            uv:       [u, 1.0],
        });
    }

    out
}
