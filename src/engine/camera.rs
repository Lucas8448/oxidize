use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // rotation around Y axis
    pub pitch: f32, // rotation around X axis
    pub fov_y: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            yaw: -90.0f32.to_radians(), // facing -Z
            pitch: 0.0,
            fov_y: 60.0f32.to_radians(),
            aspect,
            z_near: 0.1,
            z_far: 100.0,
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
    pub fn up(&self) -> Vec3 { self.right().cross(self.front()).normalize() }

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

pub struct CameraUniform { pub view_proj: Mat4 }

impl CameraUniform { pub fn new(cam: &Camera) -> Self { Self { view_proj: cam.projection_matrix() * cam.view_matrix() } } }
