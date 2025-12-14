pub const BLOCK_WORLD_VERT: &str = r#"#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aUV;
layout (location = 3) in vec4 aColor;
uniform mat4 uViewProj;
uniform mat4 uModel;
out vec4 vColor;
out vec3 vWorldPos;
out vec3 vNormal;
out vec2 vUV;
void main() {
    vec4 world = uModel * vec4(aPos, 1.0);
    vWorldPos = world.xyz;
    vNormal = aNormal;
    vColor = aColor;
    vUV = aUV;
    gl_Position = uViewProj * world;
}"#;

pub const BLOCK_WORLD_FRAG: &str = r#"#version 330 core
in vec4 vColor;
in vec3 vWorldPos;
in vec3 vNormal;
in vec2 vUV;
out vec4 FragColor;
uniform vec3 uCameraPos;
uniform sampler2D uTexture;
const vec3 LIGHT_DIR = normalize(vec3(0.4, -0.8, 0.4));
const vec3 LIGHT_COLOR = vec3(1.0, 0.98, 0.92);
const float AMBIENT = 0.45;
void main() {
    // Sample texture
    vec4 texColor = texture(uTexture, vUV);
    
    // Combine texture with vertex color (for tinting/variation)
    vec4 baseColor = texColor * vColor;
    
    // Lighting
    float diff = max(dot(normalize(vNormal), -LIGHT_DIR), 0.0);
    vec3 lit = baseColor.rgb * (LIGHT_COLOR * (AMBIENT + diff * 0.6));
    
    // Distance fog
    float dist = length(vWorldPos - uCameraPos);
    float fog = clamp((dist - 100.0) / 120.0, 0.0, 1.0);
    vec3 fogColor = vec3(0.6, 0.75, 0.95);
    vec3 finalColor = mix(lit, fogColor, fog * 0.6);
    
    FragColor = vec4(finalColor, baseColor.a);
}"#;
