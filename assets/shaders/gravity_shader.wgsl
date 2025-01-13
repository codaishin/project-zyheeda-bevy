#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::view_transformations::position_world_to_ndc
#import bevy_pbr::view_transformations::ndc_to_uv
#import bevy_pbr::mesh_view_bindings
#import bevy_render::globals::Globals
#import "shaders/helpers.wgsl"::fresnel

@group(0) @binding(11) var<uniform> globals: Globals;

@group(2) @binding(0) var first_pass_texture: texture_2d<f32>;
@group(2) @binding(1) var first_pass_sampler: sampler;

struct PulseParams {
    speed: f32,
    frequency: f32,
}

struct FresnelParams {
    power: f32,
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pulse_params: PulseParams;
    pulse_params.speed = .6;
    pulse_params.frequency = 5.;

    let fresnel = fresnel(mesh, 0.2);
    let pulse = pulse_inwards(pulse_params, fresnel);
    let uv = offset_uv_position(mesh, pulse);
    return textureSample(first_pass_texture, first_pass_sampler, uv);
}

fn pulse_inwards(params: PulseParams, value: f32) -> f32 {
    // I am sure there is a better way to do this, but this get's the job done.
    let offset = params.frequency * (globals.time * params.speed + value);
    return 1. - abs(sin(offset));
}

fn offset_uv_position(mesh: VertexOutput, offset: f32) -> vec2<f32> {
    let v_offset = normalize(project_onto_view(mesh, mesh.world_normal));
    return position_world_to_uv(mesh.world_position.xyz + v_offset * offset);
}

fn project_onto_view(mesh: VertexOutput, v: vec3<f32>) -> vec3<f32> {
    let view_normal = normalize(view.world_position - mesh.world_position.xyz);
    let v_onto_view_plane_normal = dot(v, view_normal) * view_normal;
    return v - v_onto_view_plane_normal;
}

fn position_world_to_uv(world_position: vec3<f32>) -> vec2<f32> {
    let ndc = position_world_to_ndc(world_position).xy;
    return ndc_to_uv(ndc);
}
