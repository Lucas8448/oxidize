pub const BLOCK_WORLD_VERT: &str = r#"#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec3 aColor;
uniform mat4 uViewProj;
uniform mat4 uModel;
out vec3 vColor;
out vec3 vWorldPos;
out vec3 vNormal;
void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = aNormal;
    vColor = aColor;
    gl_Position = uViewProj * world;
}"#;

pub const BLOCK_WORLD_FRAG: &str = r#"#version 330 core
in vec3 vColor;
in vec3 vWorldPos;
in vec3 vNormal;
out vec4 FragColor;
const vec3 LIGHT_DIR = normalize(vec3(0.4, -1.0, 0.3));
const vec3 LIGHT_COLOR = vec3(1.0, 0.97, 0.9);
const float AMBIENT = 0.35;
float edge_factor(float f) {
    float dist = min(min(f, 1.0 - f), 0.5);
    return smoothstep(0.0, 0.08, dist);
}
void main() {
    vec3 fracPos = fract(vWorldPos);
    float ef = edge_factor(fracPos.x) * edge_factor(fracPos.y) * edge_factor(fracPos.z);
    vec3 base = mix(vColor * 0.3, vColor, ef);
    float diff = max(dot(normalize(vNormal), -LIGHT_DIR), 0.0);
    vec3 lit = base * (LIGHT_COLOR * (AMBIENT + diff));
    float dist = length(vWorldPos);
    float fog = clamp((dist - 60.0) / 260.0, 0.0, 1.0);
    vec3 fogColor = vec3(0.55, 0.67, 0.85);
    vec3 finalColor = mix(lit, fogColor, fog * 0.65);
    FragColor = vec4(finalColor, 1.0);
}"#;
