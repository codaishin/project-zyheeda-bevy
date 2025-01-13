#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings
#import bevy_render::globals::Globals
#import "shaders/helpers.wgsl"::fresnel

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

    let fresnel = fresnel(mesh, 0.5) * 1.2;
    let pulse = pulse_inwards(fresnel, pulse_params);
    let effect = clamp(pulse - fresnel, 0., 1.);
    return vec4(material_color.rgb, material_color.a * effect);
}

fn pulse_inwards(value: f32, params: PulseParams) -> f32 {
    let offset = params.waves * (globals.time * params.speed + value);
    return abs(sin(offset));
}
