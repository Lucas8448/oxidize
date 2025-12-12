use glam::{Mat4, Vec3};
use crate::engine::mesh::Mesh;

/// An entity with a mesh, position, rotation, and scale.
/// This is a general-purpose game object that can be rendered.
#[allow(dead_code)]
pub struct Entity {
    pub mesh: Mesh,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

#[allow(dead_code)]
impl Entity {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_euler(glam::EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z),
            self.position,
        )
    }
}
