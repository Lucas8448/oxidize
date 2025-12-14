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
pub const CAMERA_SPRINT_MULTIPLIER: f32 = 2.5;
pub const CAMERA_MOUSE_SENSITIVITY: f32 = 0.0025;

/// Chunk loading settings
pub const MAX_NEW_CHUNKS_PER_FRAME: usize = 16;
pub const DEFAULT_RENDER_DISTANCE: i32 = 6;
pub const MAX_MESH_REBUILDS_PER_FRAME: usize = 4;
pub const MAX_CHUNK_RECEIVES_PER_FRAME: usize = 8;

/// Terrain generation noise parameters (Minecraft-style)
pub mod noise {
    pub const SEED: u32 = 12345;
    
    // World height settings
    pub const SEA_LEVEL: i32 = 20;
    pub const BASE_HEIGHT: i32 = 35;  // Average terrain height (above sea level)
    
    // Main terrain noise - balanced hills
    pub const TERRAIN_SCALE: f64 = 0.005;     // Middle ground scale
    pub const TERRAIN_OCTAVES: usize = 5;
    pub const TERRAIN_PERSISTENCE: f64 = 0.5;
    pub const TERRAIN_HEIGHT: f64 = 55.0;     // Moderate height variation
    
    // Detail noise (smaller bumps)
    pub const DETAIL_SCALE: f64 = 0.03;
    pub const DETAIL_HEIGHT: f64 = 5.0;
    
    // 3D cave noise (swiss cheese - big caverns, less common)
    pub const CAVE_SCALE: f64 = 0.025;
    pub const CAVE_THRESHOLD: f64 = 0.65;     // Higher = rarer but creates large caverns
    pub const CAVE_MIN_Y: i32 = 2;
    
    // Spaghetti cave noise (winding tunnels - main cave system)
    pub const SPAGHETTI_SCALE: f64 = 0.03;    // Tighter scale for more winding
    pub const SPAGHETTI_THRESHOLD: f64 = 0.055; // Slightly wider tunnels
    
    // Second spaghetti layer for more connectivity
    pub const SPAGHETTI2_SCALE: f64 = 0.02;   // Different scale for variety
    pub const SPAGHETTI2_THRESHOLD: f64 = 0.05;
    
    // Block layer depths
    pub const DIRT_DEPTH: i32 = 4;
    pub const BEDROCK_LAYERS: i32 = 3;        // Thinner bedrock layer
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
    pub const GRAVEL: u8 = 7;
}
