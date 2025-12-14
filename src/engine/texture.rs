use gl::types::*;
use std::path::Path;

/// OpenGL texture wrapper
pub struct Texture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Load a texture from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to load texture: {}", e))?
            .flipv() // OpenGL expects bottom-left origin
            .to_rgba8();
        
        let (width, height) = img.dimensions();
        let data = img.into_raw();
        
        Self::from_rgba(&data, width, height)
    }
    
    /// Create a texture from raw RGBA data
    pub fn from_rgba(data: &[u8], width: u32, height: u32) -> Result<Self, String> {
        let mut id: u32 = 0;
        
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            
            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );
            
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        
        Ok(Self { id, width, height })
    }
    
    /// Bind this texture to a texture unit
    pub unsafe fn bind(&self, unit: u32) {
        gl::ActiveTexture(gl::TEXTURE0 + unit);
        gl::BindTexture(gl::TEXTURE_2D, self.id);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

/// Generates a procedural texture atlas for blocks
/// Layout: 16x16 grid, each tile is 16x16 pixels = 256x256 total
/// Row 0: grass_top, grass_side, dirt, stone, bedrock, water, sand, ...
pub fn generate_block_atlas() -> Texture {
    const TILE_SIZE: u32 = 16;
    const ATLAS_TILES: u32 = 16;
    const ATLAS_SIZE: u32 = TILE_SIZE * ATLAS_TILES;
    
    let mut data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];
    
    // Helper to set a pixel in the atlas
    let set_pixel = |data: &mut [u8], tile_x: u32, tile_y: u32, px: u32, py: u32, r: u8, g: u8, b: u8, a: u8| {
        let x = tile_x * TILE_SIZE + px;
        let y = tile_y * TILE_SIZE + py;
        let idx = ((y * ATLAS_SIZE + x) * 4) as usize;
        data[idx] = r;
        data[idx + 1] = g;
        data[idx + 2] = b;
        data[idx + 3] = a;
    };
    
    // Simple hash for pseudo-random variation
    let hash = |x: u32, y: u32, seed: u32| -> u8 {
        let n = x.wrapping_mul(374761393)
            .wrapping_add(y.wrapping_mul(668265263))
            .wrapping_add(seed.wrapping_mul(1013904223));
        ((n >> 13) ^ n) as u8
    };
    
    // Tile 0: Grass top
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 0) as i32 - 128;
            let r = (90 + var / 8).clamp(0, 255) as u8;
            let g = (180 + var / 4).clamp(0, 255) as u8;
            let b = (70 + var / 10).clamp(0, 255) as u8;
            set_pixel(&mut data, 0, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 1: Grass side (dirt with grass edge on top)
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 1) as i32 - 128;
            if py < 4 {
                // Grass part at top
                let r = (90 + var / 8).clamp(0, 255) as u8;
                let g = (160 + var / 4).clamp(0, 255) as u8;
                let b = (60 + var / 10).clamp(0, 255) as u8;
                set_pixel(&mut data, 1, 0, px, py, r, g, b, 255);
            } else {
                // Dirt part
                let r = (140 + var / 6).clamp(0, 255) as u8;
                let g = (100 + var / 8).clamp(0, 255) as u8;
                let b = (65 + var / 10).clamp(0, 255) as u8;
                set_pixel(&mut data, 1, 0, px, py, r, g, b, 255);
            }
        }
    }
    
    // Tile 2: Dirt
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 2) as i32 - 128;
            let r = (140 + var / 6).clamp(0, 255) as u8;
            let g = (100 + var / 8).clamp(0, 255) as u8;
            let b = (65 + var / 10).clamp(0, 255) as u8;
            set_pixel(&mut data, 2, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 3: Stone
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 3) as i32 - 128;
            // Add some larger noise for stone texture
            let var2 = hash(px / 3, py / 3, 33) as i32 - 128;
            let base = 128 + var / 8 + var2 / 6;
            let r = base.clamp(0, 255) as u8;
            let g = (base - 5).clamp(0, 255) as u8;
            let b = (base - 3).clamp(0, 255) as u8;
            set_pixel(&mut data, 3, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 4: Bedrock
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 4) as i32 - 128;
            let var2 = hash(px / 2, py / 2, 44) as i32 - 128;
            let base = 40 + var / 12 + var2 / 8;
            let r = base.clamp(0, 255) as u8;
            let g = base.clamp(0, 255) as u8;
            let b = (base + 5).clamp(0, 255) as u8;
            set_pixel(&mut data, 4, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 5: Water (semi-transparent blue)
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 5) as i32 - 128;
            // Add wave-like pattern
            let wave = ((px as f32 * 0.5 + py as f32 * 0.3).sin() * 10.0) as i32;
            let r = (50 + var / 16 + wave / 2).clamp(0, 255) as u8;
            let g = (100 + var / 12 + wave).clamp(0, 255) as u8;
            let b = (200 + var / 8 + wave / 2).clamp(0, 255) as u8;
            set_pixel(&mut data, 5, 0, px, py, r, g, b, 160);
        }
    }
    
    // Tile 6: Sand
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 6) as i32 - 128;
            let var2 = hash(px / 2, py / 2, 66) as i32 - 128;
            let r = (220 + var / 10 + var2 / 12).clamp(0, 255) as u8;
            let g = (195 + var / 10 + var2 / 12).clamp(0, 255) as u8;
            let b = (140 + var / 8 + var2 / 10).clamp(0, 255) as u8;
            set_pixel(&mut data, 6, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 7: Gravel
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let var = hash(px, py, 7) as i32 - 128;
            let var2 = hash(px / 2, py / 2, 77) as i32 - 128;
            // Mix of gray with slight brown tint
            let base = 100 + var / 6 + var2 / 8;
            let r = (base + 5).clamp(0, 255) as u8;
            let g = base.clamp(0, 255) as u8;
            let b = (base - 5).clamp(0, 255) as u8;
            set_pixel(&mut data, 7, 0, px, py, r, g, b, 255);
        }
    }
    
    // Tile 8: Debug/missing texture (magenta checkerboard)
    for py in 0..TILE_SIZE {
        for px in 0..TILE_SIZE {
            let checker = ((px / 4) + (py / 4)) % 2 == 0;
            if checker {
                set_pixel(&mut data, 8, 0, px, py, 255, 0, 255, 255);
            } else {
                set_pixel(&mut data, 8, 0, px, py, 0, 0, 0, 255);
            }
        }
    }
    
    Texture::from_rgba(&data, ATLAS_SIZE, ATLAS_SIZE).expect("Failed to create block atlas")
}

/// UV coordinates for a tile in the atlas
/// Returns (u_min, v_min, u_max, v_max)
pub fn get_tile_uvs(tile_index: u32) -> (f32, f32, f32, f32) {
    const TILES_PER_ROW: u32 = 16;
    const TILE_UV_SIZE: f32 = 1.0 / 16.0;
    
    let tile_x = tile_index % TILES_PER_ROW;
    let tile_y = tile_index / TILES_PER_ROW;
    
    let u_min = tile_x as f32 * TILE_UV_SIZE;
    let v_min = tile_y as f32 * TILE_UV_SIZE;
    let u_max = u_min + TILE_UV_SIZE;
    let v_max = v_min + TILE_UV_SIZE;
    
    (u_min, v_min, u_max, v_max)
}

/// Block texture indices in the atlas
pub mod block_textures {
    pub const GRASS_TOP: u32 = 0;
    pub const GRASS_SIDE: u32 = 1;
    pub const DIRT: u32 = 2;
    pub const STONE: u32 = 3;
    pub const BEDROCK: u32 = 4;
    pub const WATER: u32 = 5;
    pub const SAND: u32 = 6;
    pub const GRAVEL: u32 = 7;
    pub const MISSING: u32 = 8;
}
