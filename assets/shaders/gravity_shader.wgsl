#import bevy_pbr::mesh_functions::get_world_from_local
#import bevy_pbr::mesh_functions::mesh_position_local_to_clip
#import bevy_pbr::mesh_functions::mesh_position_local_to_world
#import bevy_pbr::mesh_functions::mesh_normal_local_to_world
#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> time_secs: f32;

struct PulseParams {
    speed: f32,
    waves: f32,
}

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
    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pulse_params: PulseParams;
    pulse_params.speed = 0.6;
    pulse_params.waves = 3.;

    var distort_params: DistortParams;
    distort_params.falloff = 8.;
    distort_params.intensity = 2.;

    var fresnel = fresnel(mesh);
    let pulse = pulse_inwards(fresnel, pulse_params);
    fresnel = distort(fresnel, distort_params);

    let alpha = mix(pulse, fresnel, 2. / 3.);
    return vec4(material_color.rgb, alpha);
}

fn fresnel(mesh: VertexOutput) -> f32 {
    // concept taken from fresnel example in https://github.com/rust-adventure/bevy-examples
    let normal = normalize(mesh.world_normal);
    let view_vector = normalize(view.world_position.xyz - mesh.world_position.xyz);
    let normalized_angle = dot(normal, view_vector);
    return 1. - normalized_angle;
}

fn pulse_inwards(value: f32, params: PulseParams) -> f32 {
    // I am sure there is a bettwe way to do this, but this get's the job done.
    let offset = params.waves * (time_secs * params.speed + value);
    return abs(sin(offset));
}

fn distort(value: f32, params: DistortParams) -> f32 {
    let distorted = pow(value, params.falloff) * params.intensity;
    return clamp(distorted, 0., 1.);
}
