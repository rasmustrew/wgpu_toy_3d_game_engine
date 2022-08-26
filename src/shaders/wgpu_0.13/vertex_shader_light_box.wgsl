// Vertex shader


struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
};
struct Lights {
    lights: array<Light>
}
@group(1) @binding(0)
var<storage, read> lights: Lights;


struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @builtin(instance_index) index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let scale = 0.25;
    let test = u32(3);
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * scale + lights.lights[instance.index].position, 1.0);
    out.color = lights.lights[instance.index].color;
    return out;
}
