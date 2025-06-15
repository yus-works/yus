use glam::Vec3;

pub struct Camera {
    pub target: Vec3,
    pub up: Vec3,
    pub distance: f32,
    pub yaw: f32,   // in radians
    pub pitch: f32, // in radians
}

impl Camera {
    /// Current eye position derived from yaw / pitch / distance
    pub fn eye(&self) -> Vec3 {
        Vec3::new(
            self.distance * self.yaw.cos() * self.pitch.cos(),
            self.distance * self.pitch.sin(),
            self.distance * self.yaw.sin() * self.pitch.cos(),
        ) + self.target
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO, // Looking at the origin
            distance: 5.0,      // 5 units away from target
            yaw: 0.0,           // Facing along +Z by default
            pitch: 0.0,

            up: Vec3::Y, // World "up" is positive Y
        }
    }
}
