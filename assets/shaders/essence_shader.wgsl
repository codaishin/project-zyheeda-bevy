#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::mesh_functions::mesh_position_local_to_clip
#import bevy_pbr::mesh_functions::mesh_position_local_to_world
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> fill_color: vec4<f32>;
@group(2) @binding(2) var<uniform> shine_color: vec4<f32>;
@group(2) @binding(3) var material_color_texture: texture_2d<f32>;
@group(2) @binding(4) var material_color_sampler: sampler;

struct DistortParams {
    falloff: f32,
    intensity: f32,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world = get_world_from_local(vertex.instance_index);
    let vertex_position = vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.position = mesh_position_local_to_clip(world, vertex_position);
    out.world_position = mesh_position_local_to_world(world, vertex_position);
    out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    out.uv = vertex.uv;
    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var distort_params: DistortParams;
    distort_params.falloff = 15.;
    distort_params.intensity = 4.;

    let fresnel = distort(fresnel(mesh), distort_params);
    var color = material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv);
    color = mix(color, fill_color, 1. - color.a);

    return mix(color, shine_color, fresnel);
}

fn fresnel(mesh: VertexOutput) -> f32 {
    // concept taken from fresnel example in https://github.com/rust-adventure/bevy-examples
    let normal = normalize(mesh.world_normal);
    let view_vector = normalize(view.world_position.xyz - mesh.world_position.xyz);
    let normalized_angle = dot(normal, view_vector);
    return clamp(1.0 - normalized_angle, 0.0, 1.0);
}

fn distort(value: f32, params: DistortParams) -> f32 {
    return pow(value, params.falloff) * params.intensity;
}
