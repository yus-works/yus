use super::camera::Camera;

pub struct CameraInput {
    pub camera: Camera,
    pub dragging: bool,
    pub last_mouse_pos: (f32, f32),
}

impl Default for CameraInput {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            dragging: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }
}
