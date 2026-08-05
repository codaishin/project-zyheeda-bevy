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

// applied in comparison of distance to los distance
const BIAS: f32 = 0.01;

// applied in comparison of distance to los distance for decreasing ndl
const LOW_NDL_BIAS: f32 = 0.9;

// ratio of kernel area that needs to be in light for a vertex to be considered fully visible
const IN_LIGHT_UPPER_BOUND = 0.2;

const KERNEL_COUNT: u32 = 8;
const KERNEL: array<vec2<f32>, KERNEL_COUNT> = array(
    normalize(vec2(-1, -1)),
    normalize(vec2(-1,  0)),
    normalize(vec2(-1,  1)),
    normalize(vec2( 0, -1)),
    normalize(vec2( 0,  1)),
    normalize(vec2( 1, -1)),
    normalize(vec2( 1,  0)),
    normalize(vec2( 1,  1)),
);
const CONE_SAMPLE_COUNT = 10;
const CONE_RADIUS: f32 = 0.002;

struct VecSample {
    x: vec3<f32>,
    y: vec3<f32>,
}

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
#endif

#ifdef NO_LIGHT
    out.color = vec4(vec3(0), out.color.a);
#endif

#ifdef LIGHT
    let light = compute_light(in);
    out.color = vec4(out.color.rgb * light, out.color.a);
#endif

    return out;
}

fn compute_light(in: VertexOutput) -> f32 {
    var direction = in.world_position.xyz - player_position;
    let vertex_distance = length(direction);

    if vertex_distance == 0 {
        return final_light(1, vertex_distance, 1);
    }

    if vertex_distance > range {
        return final_light(0, vertex_distance, 1);
    }

    direction = normalize(direction);

#ifdef IGNORE_NDL
    var ndl = 1.0;
#else
    var ndl = max(dot(-direction, normalize(in.world_normal.xyz)), 0.0);
#endif
    let bias = mix(LOW_NDL_BIAS, BIAS, ndl);
    let los_distance = get_los_distance(direction);

    var in_light = compute_visibility(los_distance, vertex_distance, bias);

    if in_light == 1 {
        return final_light(1, vertex_distance, ndl);
    }

    let sample_vectors = compute_sample_vectors(direction);
    for (var c: u32 = 0; c < CONE_SAMPLE_COUNT; c++) {
        let radius = CONE_RADIUS * f32(c + 1);
        for (var k: u32 = 0; k < KERNEL_COUNT; k++) {
            let kernel = KERNEL[k];
            let offset = sample_vectors.x * kernel.x + sample_vectors.y * kernel.y;
            let sample_direction = normalize(direction + offset * radius);
            let sample_los_distance = get_los_distance(sample_direction);

            in_light += compute_visibility(sample_los_distance, vertex_distance, bias);
        }
    }
    in_light /= f32(KERNEL_COUNT * CONE_SAMPLE_COUNT + 1);
    let visibility = smoothstep(0.0, IN_LIGHT_UPPER_BOUND, in_light);

    return final_light(visibility, vertex_distance, ndl);
}

fn get_los_distance(direction: vec3<f32>) -> f32 {
    return textureSample(
        los_texture,
        los_sampler,
        direction * CUBEMAP_SAMPLE_CORRECTION
    ).r * range;
}

fn compute_sample_vectors(direction: vec3<f32>) -> VecSample {
    var y = select(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        dot(direction, vec3(0, 1, 0)) < 0.9
    );
    let x = cross(direction, y);
    y = cross(direction, x);

    return VecSample(x, y);
}

fn compute_visibility(los_distance: f32, distance: f32, bias: f32) -> f32 {
    return smoothstep(-bias, bias, los_distance - distance);
}

fn final_light(visibility: f32, distance: f32, ndl: f32) -> f32 {
    let normalized_distance = distance / range;
    let light = max(1. - normalized_distance, min_light);
    let scaled_ndl = pow(1 - ndl, 5);
    let ndl_weight = select(scaled_ndl, 0, visibility == 0);
    let visibility_or_ndl = mix(visibility, ndl, ndl_weight);

    return mix(min_light, light, visibility_or_ndl);
}
