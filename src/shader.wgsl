struct Uniforms { mvp: mat4x4<f32>, };
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var terrain_tex: texture_2d<f32>;
@group(0) @binding(2) var terrain_sampler: sampler;

struct VSOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VSOut {
    var out: VSOut;
    out.pos = uniforms.mvp * vec4<f32>(position, 1.0);
    out.normal = normal;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    let base = textureSample(terrain_tex, terrain_sampler, in.uv);
    return base;
}