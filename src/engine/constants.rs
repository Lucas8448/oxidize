/// Default window dimensions
pub const DEFAULT_WINDOW_WIDTH: u32 = 1600; 
pub const DEFAULT_WINDOW_HEIGHT: u32 = 1200;

/// Sky blue clear color (RGBA)
pub const CLEAR_COLOR: (f32, f32, f32, f32) = (0.4, 0.6, 0.9, 1.0);

/// Camera configuration
pub const CAMERA_FOV_Y_DEGREES: f32 = 60.0;
pub const CAMERA_Z_NEAR: f32 = 0.1;
pub const CAMERA_Z_FAR: f32 = 500.0;
pub const CAMERA_MOVE_SPEED: f32 = 8.0;
pub const CAMERA_MOUSE_SENSITIVITY: f32 = 0.0025;

/// Chunk loading settings
pub const MAX_NEW_CHUNKS_PER_FRAME: usize = 12;
pub const DEFAULT_RENDER_DISTANCE: i32 = 4;

/// Terrain generation noise parameters
pub mod noise {
    pub const SEED: u32 = 0;
    pub const BASE_SCALE: f64 = 0.015;
    pub const OCTAVES: usize = 4;
    pub const PERSISTENCE: f32 = 0.55;
    pub const LACUNARITY: f32 = 2.1;
    pub const MAX_HEIGHT: f32 = 38.0;
    pub const HEIGHT_EXPONENT: f32 = 1.2;
    pub const DIRT_DEPTH: usize = 3;
}

/// Block type identifiers
#[allow(dead_code)]
pub mod blocks {
    pub const GRASS: u8 = 1;
    pub const DIRT: u8 = 2;
    pub const STONE: u8 = 3;
    pub const BEDROCK: u8 = 4;
    pub const WATER: u8 = 5;
    pub const SAND: u8 = 6;
}
