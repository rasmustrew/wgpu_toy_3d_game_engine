

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tangent_position: vec3<f32>,
    @location(2) tangent_view_position: vec3<f32>,
    @location(3) tangent_matrix_1: vec3<f32>,
    @location(4) tangent_matrix_2: vec3<f32>,
    @location(5) tangent_matrix_3: vec3<f32>,
    
};

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
};
struct Lights {
    lights: array<Light>
}
@group(2) @binding(0)
var<storage, read> lights: Lights;



// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {

    let tangent_matrix = mat3x3<f32>(
        in.tangent_matrix_1,
        in.tangent_matrix_2,
        in.tangent_matrix_3,
    ); 

    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    //Calculate ambient light as average color of all lights
    let num_lights = arrayLength(&lights.lights);
    
    var combined_light_color = vec3(0.0, 0.0, 0.0);

    for (var i = 0; i < i32(num_lights); i=i+1) {
        let tangent_light_position = tangent_matrix * lights.lights[i].position;
        let ambient_strength = 0.1;
        let ambient_color = lights.lights[i].color * ambient_strength;

        let tangent_normal = object_normal.xyz * 2.0 - 1.0;
        let light_dir = normalize(tangent_light_position - in.tangent_position);
        let view_dir = normalize(in.tangent_view_position - in.tangent_position);

        // Diffuse
        let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
        let diffuse_color = lights.lights[i].color * diffuse_strength;

        // Specular
        let half_dir = normalize(view_dir + light_dir);
        let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * lights.lights[i].color;

        combined_light_color += ambient_color + diffuse_color + specular_color;
    }

    combined_light_color = combined_light_color / f32(num_lights);
    let result = combined_light_color * object_color.xyz;
    


    return vec4<f32>(result, object_color.a);
}
