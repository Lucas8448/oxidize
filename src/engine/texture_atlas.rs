pub struct AtlasResources {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub fn build_terrain_atlas(device: &wgpu::Device, queue: &wgpu::Queue) -> AtlasResources {
    let tile_px: u32 = 32;
    let tiles_w: u32 = 4;
    let tex_size = wgpu::Extent3d { width: tile_px * tiles_w, height: tile_px * tiles_w, depth_or_array_layers: 1 };
    let solid_colors = [
        [34u8, 139, 34, 255],
        [134,  96,  67, 255],
        [120, 120, 120, 255],
        [0,0,0,0],
    ];
    let mut data = vec![0u8; (tex_size.width * tex_size.height * 4) as usize];
    for ty in 0..tiles_w { for tx in 0..tiles_w { for py in 0..tile_px { for px in 0..tile_px {
        let x = tx * tile_px + px; let y = ty * tile_px + py; let idx = ((y * tex_size.width + x) * 4) as usize;
        let rgba = if ty == 0 { match tx {
            0 => solid_colors[0], 1 => solid_colors[1], 2 => solid_colors[2], 3 => {
                let cap_px = 1; let blend_px = 4; if py < cap_px { [44,160,44,255] } else if py < blend_px {
                    let t = (py - cap_px) as f32 / (blend_px - cap_px) as f32;
                    let g = glam::Vec4::from_array([34.0,139.0,34.0,255.0]);
                    let d = glam::Vec4::from_array([134.0,96.0,67.0,255.0]);
                    let mix = g.lerp(d, t); [mix.x as u8, mix.y as u8, mix.z as u8, 255]
                } else { solid_colors[1] } }
            _ => [255,0,255,255], } } else { [50 + (tx*40) as u8, 50 + (ty*40) as u8, 150, 255] };
        data[idx..idx+4].copy_from_slice(&rgba);
    } } } }
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("TerrainAtlas"), size: tex_size, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        &data,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(tex_size.width * 4), rows_per_image: Some(tex_size.height) },
        tex_size,
    );
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("TerrainSampler"), address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge, address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest, min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest, ..Default::default()
    });
    AtlasResources { texture_view, sampler }
}
