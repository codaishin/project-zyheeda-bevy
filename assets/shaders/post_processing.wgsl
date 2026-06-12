#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var world_depth: texture_depth_2d;
@group(0) @binding(1) var world_depth_sampler: sampler_comparison;
@group(0) @binding(2) var outline_depth: texture_depth_2d;
@group(0) @binding(3) var outline_depth_sampler: sampler_comparison;
@group(0) @binding(4) var screen_texture: texture_2d<f32>;
@group(0) @binding(5) var screen_texture_sampler: sampler;
@group(0) @binding(6) var outline_texture: texture_2d<f32>;
@group(0) @binding(7) var outline_texture_sampler: sampler;
@group(0) @binding(8) var<uniform> outline_settings: PostProcessSettings;

struct PostProcessSettings {
    outline_color: vec4<f32>,
}

const OFFSETS = array(
    vec2( 1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2(-1.0, -1.0),
);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let screen = textureSample(screen_texture, screen_texture_sampler, in.uv);
    let probe_offset = 2.0 / vec2<f32>(textureDimensions(outline_texture));

    var probe = textureSample(outline_texture, outline_texture_sampler, in.uv).a;

    if probe != 0.0 {
        return screen;
    }

    for (var i = 0; i < 4; i++) {
        let probe_uv = in.uv + OFFSETS[i] * probe_offset;

        probe = textureSample(outline_texture, outline_texture_sampler, probe_uv).a;

        if probe != 0.0 {
            return outline_settings.outline_color;
        }
    }

    return screen;
}
