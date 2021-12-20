

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] tangent_position: vec3<f32>;
    [[location(2)]] tangent_light_position: vec3<f32>;
    [[location(3)]] tangent_view_position: vec3<f32>;
    
};

struct LightPosition {
    position: vec3<f32>;
};
struct LightColor {
    color: vec3<f32>;
};
[[group(2), binding(0)]]
var<uniform> light_position: LightPosition;
[[group(2), binding(1)]]
var<uniform> light_color: LightColor;


// Fragment shader
[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
[[group(0), binding(2)]]
var t_normal: texture_2d<f32>;
[[group(0), binding(3)]]
var s_normal: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light_color.color * ambient_strength;

    // Create the lighting vectors
    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir = normalize(in.tangent_light_position - in.tangent_position);
    let view_dir = normalize(in.tangent_view_position - in.tangent_position);

    // Diffuse
    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light_color.color * diffuse_strength;

    // Specular
    let half_dir = normalize(view_dir + light_dir);
    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light_color.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;


    return vec4<f32>(result, object_color.a);
}
