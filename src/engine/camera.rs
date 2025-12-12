use glam::{Mat4, Vec3, Vec4};
use crate::engine::constants::{CAMERA_FOV_Y_DEGREES, CAMERA_Z_NEAR, CAMERA_Z_FAR};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov_y: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Debug)]
pub struct Frustum {
    pub planes: [Vec4; 6],
}

impl Frustum {
    pub fn contains_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let n = plane.truncate();
            let d = plane.w;
            let positive_vertex = Vec3::new(
                if n.x >= 0.0 { max.x } else { min.x },
                if n.y >= 0.0 { max.y } else { min.y },
                if n.z >= 0.0 { max.z } else { min.z },
            );
            if n.dot(positive_vertex) + d < 0.0 {
                return false;
            }
        }
        true
    }
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 45.0, 0.0),
            yaw: -90.0f32.to_radians(),
            pitch: 0.0,
            fov_y: CAMERA_FOV_Y_DEGREES.to_radians(),
            aspect,
            z_near: CAMERA_Z_NEAR,
            z_far: CAMERA_Z_FAR,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let front = self.front();
        Mat4::look_at_rh(self.position, self.position + front, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far)
    }

    pub fn front(&self) -> Vec3 {
        let x = self.yaw.cos() * self.pitch.cos();
        let y = self.pitch.sin();
        let z = self.yaw.sin() * self.pitch.cos();
        Vec3::new(x, y, z).normalize()
    }

    pub fn right(&self) -> Vec3 { self.front().cross(Vec3::Y).normalize() }

    #[allow(dead_code)]
    pub fn up(&self) -> Vec3 { self.right().cross(self.front()).normalize() }

    pub fn frustum(&self) -> Frustum {
        let view_proj = self.projection_matrix() * self.view_matrix();
        let m = view_proj.to_cols_array();
        let left   = Vec4::new(m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]).normalize();
        let right  = Vec4::new(m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]).normalize();
        let bottom = Vec4::new(m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]).normalize();
        let top    = Vec4::new(m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]).normalize();
        let near   = Vec4::new(m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]).normalize();
        let far    = Vec4::new(m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]).normalize();
        Frustum { planes: [left, right, bottom, top, near, far] }
    }

    pub fn process_keyboard(&mut self, direction: CameraMove, delta: f32) {
        let speed = 2.5 * delta;
        match direction {
            CameraMove::Forward => self.position += self.front() * speed,
            CameraMove::Backward => self.position -= self.front() * speed,
            CameraMove::Left => self.position -= self.right() * speed,
            CameraMove::Right => self.position += self.right() * speed,
            CameraMove::Up => self.position += Vec3::Y * speed,
            CameraMove::Down => self.position -= Vec3::Y * speed,
        }
    }

    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.0025;
        self.yaw += dx * sensitivity;
        self.pitch -= dy * sensitivity;
        let max_pitch = 89.0f32.to_radians();
        if self.pitch > max_pitch { self.pitch = max_pitch; }
        if self.pitch < -max_pitch { self.pitch = -max_pitch; }
    }
}

pub enum CameraMove { Forward, Backward, Left, Right, Up, Down }

#[allow(dead_code)]
pub struct CameraUniform {
    pub view_proj: Mat4,
}

#[allow(dead_code)]
impl CameraUniform {
    pub fn new(cam: &Camera) -> Self {
        Self { view_proj: cam.projection_matrix() * cam.view_matrix() }
    }
}
