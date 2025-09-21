use std::collections::{HashMap, HashSet};
use winit::{dpi::PhysicalSize, window::Window};
use wgpu::util::DeviceExt;
use crate::world::{World, build_chunk_mesh, ChunkMesh, Vertex, CHUNK_SIZE};
use super::{camera::Camera, input::InputState, texture_atlas::build_terrain_atlas};
use winit::keyboard::{Key, NamedKey};
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::{ActiveEventLoop};
use winit::application::ApplicationHandler;
use winit::window::WindowId;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms { mvp: [[f32;4];4] }

struct LoadedChunk { vertex_buf: wgpu::Buffer, index_buf: wgpu::Buffer, index_count: u32 }

pub struct EngineState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    world: World,
    loaded: HashMap<(i32,i32), LoadedChunk>,
    draw_radius: i32,
    buffer_radius: i32,
    last_camera_chunk: (i32,i32),
    uniform_buf: wgpu::Buffer,
    camera: Camera,
    input: InputState,
    last_instant: std::time::Instant,
}

impl<'a> EngineState<'a> {
    async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.unwrap();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration { usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: size.width.max(1), height: size.height.max(1), present_mode: caps.present_modes[0], alpha_mode: caps.alpha_modes[0], view_formats: vec![], desired_maximum_frame_latency: 2 };
        surface.configure(&device, &config);

        let mut world = World::new(1337);
        let draw_radius = 4; let buffer_radius = draw_radius + 1;
        for cz in -buffer_radius..=buffer_radius { for cx in -buffer_radius..=buffer_radius { world.ensure_chunk(cx,cz); } }
        let mut loaded: HashMap<(i32,i32), LoadedChunk> = HashMap::new();
        for cz in -buffer_radius..=buffer_radius { for cx in -buffer_radius..=buffer_radius { let getter = |qx: i32, qz: i32| world.get_chunk(qx,qz); let ChunkMesh { vertices, indices } = build_chunk_mesh(&getter, cx, cz); if vertices.is_empty() { continue; } let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Vertex", cx, cz)), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX }); let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Index", cx, cz)), contents: bytemuck::cast_slice(&indices), usage: wgpu::BufferUsages::INDEX }); loaded.insert((cx,cz), LoadedChunk { vertex_buf: vbuf, index_buf: ibuf, index_count: indices.len() as u32 }); } }
        let last_camera_chunk = (0,0);

        let camera = Camera::new_perspective(config.width as f32 / config.height as f32);
        let mvp = camera.view_proj();
        let uniforms = Uniforms { mvp: mvp.to_cols_array_2d() };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("Uniform Buffer"), contents: bytemuck::bytes_of(&uniforms), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });

        let atlas = build_terrain_atlas(&device, &queue);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Shader"), source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()) });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("Scene Layout"), entries: &[
            wgpu::BindGroupLayoutEntry { binding:0, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding:1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None },
            wgpu::BindGroupLayoutEntry { binding:2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
        ] });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Scene Bind Group"), layout: &bind_layout, entries: &[
            wgpu::BindGroupEntry { binding:0, resource: uniform_buf.as_entire_binding() },
            wgpu::BindGroupEntry { binding:1, resource: wgpu::BindingResource::TextureView(&atlas.texture_view) },
            wgpu::BindGroupEntry { binding:2, resource: wgpu::BindingResource::Sampler(&atlas.sampler) },
        ] });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Pipeline Layout"), bind_group_layouts: &[&bind_layout], push_constant_ranges: &[] });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { label: Some("Render Pipeline"), layout: Some(&pipeline_layout), vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[Vertex::layout()], compilation_options: wgpu::PipelineCompilationOptions::default() }, fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })], compilation_options: wgpu::PipelineCompilationOptions::default() }), primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() }, depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None, cache: None });

        Self { surface, device, queue, config, size, pipeline, bind_group, world, loaded, draw_radius, buffer_radius, last_camera_chunk, uniform_buf, camera, input: InputState::new(), last_instant: std::time::Instant::now() }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) { self.size = new_size; self.config.width = new_size.width.max(1); self.config.height = new_size.height.max(1); self.surface.configure(&self.device, &self.config); }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.update_camera(); self.update_streaming();
        let frame = self.surface.get_current_texture()?; let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        { let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("Main Pass"), color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r:0.53,g:0.81,b:0.92,a:1.0 }), store: wgpu::StoreOp::Store }, depth_slice: None })], depth_stencil_attachment: None, occlusion_query_set: None, timestamp_writes: None });
            pass.set_pipeline(&self.pipeline); pass.set_bind_group(0, &self.bind_group, &[]);
            for (&(cx,cz), chunk) in self.loaded.iter() { let (ccx, ccz) = self.last_camera_chunk; if (cx - ccx).abs() > self.draw_radius || (cz - ccz).abs() > self.draw_radius { continue; } pass.set_vertex_buffer(0, chunk.vertex_buf.slice(..)); pass.set_index_buffer(chunk.index_buf.slice(..), wgpu::IndexFormat::Uint32); pass.draw_indexed(0..chunk.index_count, 0, 0..1); }
        }
        self.queue.submit(Some(encoder.finish())); frame.present(); Ok(())
    }

    fn update_camera(&mut self) {
        let now = std::time::Instant::now(); let dt = (now - self.last_instant).as_secs_f32(); self.last_instant = now;
        let speed = 20.0 * dt; let rot_speed = 90f32.to_radians() * dt;
        let forward = glam::Vec3::new(self.camera.yaw.cos() * self.camera.pitch.cos(), self.camera.pitch.sin(), self.camera.yaw.sin() * self.camera.pitch.cos()).normalize();
        let right = forward.cross(glam::Vec3::Y).normalize(); let up = glam::Vec3::Y;
        if self.input.is("w") { self.camera.position += forward * speed; }
        if self.input.is("s") { self.camera.position -= forward * speed; }
        if self.input.is("a") { self.camera.position -= right * speed; }
        if self.input.is("d") { self.camera.position += right * speed; }
        if self.input.is("space") { self.camera.position += up * speed; }
        if self.input.is("shift") { self.camera.position -= up * speed; }
        if self.input.is("arrowleft") { self.camera.yaw -= rot_speed; }
        if self.input.is("arrowright") { self.camera.yaw += rot_speed; }
        if self.input.is("arrowup") { self.camera.pitch += rot_speed; }
        if self.input.is("arrowdown") { self.camera.pitch -= rot_speed; }
        let max_pitch = 89f32.to_radians(); if self.camera.pitch > max_pitch { self.camera.pitch = max_pitch; } if self.camera.pitch < -max_pitch { self.camera.pitch = -max_pitch; }
        let mvp = self.camera.view_proj(); let uniforms = Uniforms { mvp: mvp.to_cols_array_2d() }; self.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));
    }

    fn update_streaming(&mut self) {
        let cx = (self.camera.position.x.floor() as i32).div_euclid(CHUNK_SIZE as i32); let cz = (self.camera.position.z.floor() as i32).div_euclid(CHUNK_SIZE as i32); let current = (cx,cz); if current == self.last_camera_chunk { return; } self.last_camera_chunk = current;
        let mut desired = Vec::new(); for dz in -self.buffer_radius..=self.buffer_radius { for dx in -self.buffer_radius..=self.buffer_radius { desired.push((cx+dx, cz+dz)); } }
        for &(x,z) in &desired { self.world.ensure_chunk(x,z); }
        let getter = |qx: i32, qz: i32| self.world.get_chunk(qx,qz);
        let mut newly_loaded = Vec::new();
        for &(x,z) in &desired { if !self.loaded.contains_key(&(x,z)) { let mesh = build_chunk_mesh(&getter, x, z); if mesh.vertices.is_empty() { continue; } let vbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Vertex", x, z)), contents: bytemuck::cast_slice(&mesh.vertices), usage: wgpu::BufferUsages::VERTEX }); let ibuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Index", x, z)), contents: bytemuck::cast_slice(&mesh.indices), usage: wgpu::BufferUsages::INDEX }); self.loaded.insert((x,z), LoadedChunk { vertex_buf: vbuf, index_buf: ibuf, index_count: mesh.indices.len() as u32 }); newly_loaded.push((x,z)); } }
        let mut to_remove = Vec::new(); for (&(x,z), _) in self.loaded.iter() { if (x - cx).abs() > self.buffer_radius || (z - cz).abs() > self.buffer_radius { to_remove.push((x,z)); } }
        for key in &to_remove { self.loaded.remove(key); }
        let mut need_rebuild = HashSet::new(); let dirs = [(1,0),(-1,0),(0,1),(0,-1)]; for &(x,z) in &newly_loaded { for (dx,dz) in dirs { need_rebuild.insert((x+dx,z+dz)); } } for &(x,z) in &to_remove { for (dx,dz) in dirs { need_rebuild.insert((x+dx,z+dz)); } }
        need_rebuild.retain(|pos| self.loaded.contains_key(pos)); for (x,z) in need_rebuild.into_iter() { let mesh = build_chunk_mesh(&getter, x, z); let vbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Vertex", x, z)), contents: bytemuck::cast_slice(&mesh.vertices), usage: wgpu::BufferUsages::VERTEX }); let ibuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(&format!("Chunk({}, {}) Index", x, z)), contents: bytemuck::cast_slice(&mesh.indices), usage: wgpu::BufferUsages::INDEX }); if let Some(entry) = self.loaded.get_mut(&(x,z)) { entry.vertex_buf = vbuf; entry.index_buf = ibuf; entry.index_count = mesh.indices.len() as u32; } }
    }
}

pub struct App { state: Option<EngineState<'static>>, window: Option<&'static Window> }
impl App { pub fn new() -> Self { Self { state: None, window: None } } }
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) { if self.window.is_none() { let window = el.create_window(Window::default_attributes().with_title("Oxidize Voxel")).unwrap(); let window = Box::leak(Box::new(window)); let state = pollster::block_on(EngineState::new(window)); self.window = Some(window); self.state = Some(state); window.request_redraw(); } }
    fn window_event(&mut self, _el: &ActiveEventLoop, id: WindowId, event: WindowEvent) { if let (Some(window), Some(state)) = (self.window, self.state.as_mut()) { if id == window.id() { match event { WindowEvent::CloseRequested => std::process::exit(0), WindowEvent::Resized(size) => state.resize(size), WindowEvent::RedrawRequested => { let _ = state.render(); window.request_redraw(); }, WindowEvent::KeyboardInput { event: KeyEvent { logical_key: key, state: key_state, .. }, .. } => { let key_id = match key { Key::Named(nk) => match nk { NamedKey::Space => "space".to_string(), NamedKey::Shift => "shift".to_string(), NamedKey::ArrowLeft => "arrowleft".to_string(), NamedKey::ArrowRight => "arrowright".to_string(), NamedKey::ArrowUp => "arrowup".to_string(), NamedKey::ArrowDown => "arrowdown".to_string(), _ => return }, Key::Character(c) => c.to_lowercase(), _ => return }; match key_state { ElementState::Pressed => { state.input.press(key_id); }, ElementState::Released => { state.input.release(&key_id); }, } }, _ => {} } } } }
}
