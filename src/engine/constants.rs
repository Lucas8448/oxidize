pub const DEFAULT_WINDOW_WIDTH: u32 = 800;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 600;

pub const CLEAR_COLOR: (f32, f32, f32, f32) = (0.4, 0.6, 0.9, 1.0);

pub const CAMERA_FOV_Y_DEGREES: f32 = 60.0;
pub const CAMERA_Z_NEAR: f32 = 0.1;
pub const CAMERA_Z_FAR: f32 = 500.0;

pub const MAX_NEW_CHUNKS_PER_FRAME: usize = 12;
pub const DEFAULT_RENDER_DISTANCE: i32 = 4;

pub mod noise {
    pub const BASE_SCALE: f64 = 0.015;
    pub const OCTAVES: usize = 4;
    pub const PERSISTENCE: f32 = 0.55;
    pub const LACUNARITY: f32 = 2.1;
    pub const MAX_HEIGHT: f32 = 38.0;
    pub const HEIGHT_EXPONENT: f32 = 1.2;
}
