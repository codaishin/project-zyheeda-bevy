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

const MAX_DIST: f32 = 1;
const OFFSETS = array(
    vec2( 1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2(-1.0, -1.0),
);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let cam_distance = cam_distance(in.uv);

    if cam_distance.outlined <= cam_distance.world {
        return screen(in.uv);
    }

    let offset = 1.1 / vec2<f32>(textureDimensions(outline_texture));
    for (var i = 0; i < 4; i++) {
        let probe_uv = in.uv + OFFSETS[i] * offset;
        let cam_distance = cam_distance(probe_uv);

        if cam_distance.world == MAX_DIST {
            continue;
        }

        if cam_distance.outlined <= cam_distance.world {
            return outline_settings.outline_color;
        }
    }

    return screen(in.uv);
}

struct CamDistance {
    world: f32,
    outlined: f32,
}

fn cam_distance(uv: vec2<f32>) -> CamDistance {
    let pos = vec2<i32>(uv * vec2<f32>(textureDimensions(world_depth).xy));
    var dist: CamDistance;

    dist.world = MAX_DIST - textureLoad(world_depth, pos, 0);
    dist.outlined = MAX_DIST - textureLoad(outline_depth, pos, 0);

    return dist;
}

fn screen(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(screen_texture, screen_texture_sampler, uv);
}
