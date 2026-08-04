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

const BIAS: f32 = 0.03;
const FLAT_ANGLE_BIAS: f32 = 0.3;

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
const CONE_SAMPLE_COUNT = 8;
const CONE_RADIUS: f32 = 0.005;

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

    #ifdef DEAD_SPACE
        out.color = vec4(vec3(0), out.color.a);
    #else
        var visibility: f32;

        var direction = in.world_position.xyz - player_position;
        let vertex_distance = length(direction);

        if vertex_distance == 0 {
            visibility = 1;
        } else if vertex_distance > range {
            visibility = 0;
        } else {
            direction = normalize(direction);

            // Compute visibility
            let los_distance = textureSample(
                los_texture,
                los_sampler,
                direction * CUBEMAP_SAMPLE_CORRECTION
            ).r * range;
            visibility = step(vertex_distance, los_distance + BIAS);

            // Cone visibility
            if visibility < 1 {
                var up = select(
                    vec3(1.0, 0.0, 0.0),
                    vec3(0.0, 1.0, 0.0),
                    dot(direction, vec3(0, 1, 0)) < 0.9
                );
                let right = cross(direction, up);
                up = cross(direction, right);

                var radius_scale = max((vertex_distance - los_distance) / range, 0.0);
                var in_light = 0.0;
                for (var c: u32 = 0; c < CONE_SAMPLE_COUNT; c++) {
                    let radius = CONE_RADIUS * radius_scale * f32(c + 1);
                    for (var k: u32 = 0; k < KERNEL_COUNT; k++) {
                        let kernel = KERNEL[k];
                        let offset = right * kernel.x + up * kernel.y;
                        let kernel_direction = normalize(direction + offset * radius);
                        let kernel_los_distance = textureSample(
                            los_texture,
                            los_sampler,
                            kernel_direction * CUBEMAP_SAMPLE_CORRECTION
                        ).r * range;

                        in_light += step(vertex_distance, kernel_los_distance + BIAS);
                    }
                }
                in_light /= f32(KERNEL_COUNT * CONE_SAMPLE_COUNT);
                visibility = smoothstep(0.0, 0.2, in_light);
            }
        }

        // Apply Visibility
        let light = mix(
            min_light,
            max(1. - (vertex_distance / range), min_light),
            visibility,
        );
        out.color = vec4(out.color.rgb * light, out.color.a);
    #endif
#endif

    return out;
}
