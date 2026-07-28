#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> player_position: vec3<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var<uniform> range: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var<uniform> min_light: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var los_texture: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var los_sampler: sampler;

// corrects for the right/left handed coordinate systems mismatch between bevy and wgsl
const CUBEMAP_SAMPLE_CORRECTION: vec3<f32> = vec3(1.0, 1.0, -1.0);

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    let direction = in.world_position.xyz - player_position;
    let vertex_distance = length(direction) / range;
    var los_distance = textureSample(
        los_texture,
        los_sampler,
        normalize(direction) * CUBEMAP_SAMPLE_CORRECTION
    ).r;

    if vertex_distance <= los_distance + 0.01 {
        out.color = vec4(out.color.rgb * max(1. - vertex_distance, min_light), out.color.a);
    } else {
        out.color = vec4(out.color.rgb * min_light, out.color.a);
    }

#endif

    return out;
}
