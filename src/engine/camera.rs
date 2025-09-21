use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub proj: Mat4,
}

impl Camera {
    pub fn new_perspective(aspect: f32) -> Self {
        let proj = Mat4::perspective_rh(60f32.to_radians(), aspect, 0.1, 500.0);
        Self {
            position: glam::vec3(8.0, 35.0, 24.0),
            yaw: -90f32.to_radians(),
            pitch: -25f32.to_radians(),
            proj,
        }
    }
    pub fn view_proj(&self) -> Mat4 {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize();
        let up = Vec3::Y;
        let target = self.position + forward;
        self.proj * Mat4::look_at_rh(self.position, target, up)
    }
}
