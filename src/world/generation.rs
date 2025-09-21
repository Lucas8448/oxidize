use super::{chunk::{Chunk, CHUNK_SIZE, CHUNK_HEIGHT}, block::Block};

pub fn generate_chunk(seed: u32, cx: i32, cz: i32) -> Chunk {
    use noise::{NoiseFn, Perlin};
    let perlin = Perlin::new(seed);
    let mut chunk = Chunk::empty();
    let mut fbm = |wx: f64, wz: f64| {
        let mut amp = 1.0;
        let mut freq = 1.0 / 64.0;
        let mut sum = 0.0;
        let mut norm = 0.0;
        for _oct in 0..3 {
            let v = perlin.get([wx * freq, wz * freq]);
            sum += v * amp;
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        (sum / norm).clamp(-1.0, 1.0)
    };

    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let world_x = (cx as i64 * CHUNK_SIZE as i64 + x as i64) as f64;
            let world_z = (cz as i64 * CHUNK_SIZE as i64 + z as i64) as f64;
            let n = fbm(world_x as f64, world_z as f64);
            let hn = (n * 0.5 + 0.5).powf(1.1);
            let base = 4.0;
            let range = (CHUNK_HEIGHT as f64 * 0.55).min((CHUNK_HEIGHT - 2) as f64);
            let mut h = (base + hn * range) as isize;
            if h < 1 { h = 1; }
            if h > (CHUNK_HEIGHT - 2) as isize { h = (CHUNK_HEIGHT - 2) as isize; }
            let h_usize = h as usize;
            let dirt_thickness = 6usize.min(h_usize);
            for y in 0..=h_usize {
                let block = if y == h_usize { Block::Grass }
                    else if y + dirt_thickness >= h_usize { Block::Dirt }
                    else { Block::Stone };
                chunk.blocks[y][z][x] = block;
            }
        }
    }
    chunk
}
