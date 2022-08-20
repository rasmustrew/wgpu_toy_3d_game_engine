// Vertex shader


struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

struct LightPosition {
    position: vec3<f32>,
};
struct LightColor {
    color: vec3<f32>,
};
@group(1) @binding(0)
var<uniform> light_position: LightPosition;
@group(1) @binding(1)
var<uniform> light_color: LightColor;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn main(
    model: VertexInput,
) -> VertexOutput {
    let scale = 0.25;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * scale + light_position.position, 1.0);
    out.color = light_color.color;
    return out;
}
