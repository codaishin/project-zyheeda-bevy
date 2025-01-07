#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_view_bindings
#import bevy_render::globals::Globals
#import "shaders/helpers.wgsl"::fresnel
#import "shaders/helpers.wgsl"::distort
#import "shaders/helpers.wgsl"::DistortParams

@group(0) @binding(11) var<uniform> globals: Globals;

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;

struct PulseParams {
    speed: f32,
    waves: f32,
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var pulse_params: PulseParams;
    pulse_params.speed = 0.6;
    pulse_params.waves = 3.;

    var distort_params: DistortParams;
    distort_params.falloff = 0.5;
    distort_params.intensity = 1.2;

    var fresnel = fresnel(mesh.world_normal, mesh.world_position, view.world_position);
    let pulse = pulse_inwards(fresnel, pulse_params);
    fresnel = distort(fresnel, distort_params);

    let effect = clamp(pulse - fresnel, 0., 1.);
    return vec4(material_color.rgb, material_color.a * effect);
}

fn pulse_inwards(value: f32, params: PulseParams) -> f32 {
    let offset = params.waves * (globals.time * params.speed + value);
    return abs(sin(offset));
}
