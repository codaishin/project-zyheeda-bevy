#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding( 0) var world_depth: texture_depth_2d;
@group(0) @binding( 1) var world_depth_sampler: sampler_comparison;
@group(0) @binding( 2) var agents_depth: texture_depth_2d;
@group(0) @binding( 3) var agents_depth_sampler: sampler_comparison;
@group(0) @binding( 4) var outline_depth: texture_depth_2d;
@group(0) @binding( 5) var outline_depth_sampler: sampler_comparison;
@group(0) @binding( 6) var screen_texture: texture_2d<f32>;
@group(0) @binding( 7) var screen_texture_sampler: sampler;
@group(0) @binding( 8) var agents_texture: texture_2d<f32>;
@group(0) @binding( 9) var agents_texture_sampler: sampler;
@group(0) @binding(10) var visibility_texture: texture_2d<f32>;
@group(0) @binding(11) var visibility_texture_sampler: sampler;
@group(0) @binding(12) var outline_texture: texture_2d<f32>;
@group(0) @binding(13) var outline_texture_sampler: sampler;
@group(0) @binding(14) var<uniform> outline_settings: PostProcessSettings;

alias Kind = u32;

struct PostProcessSettings {
    outline_color: vec4<f32>,
}

struct Depths {
    world: f32,
    agent: f32,
    outlined: f32,
}

struct InFront {
    kind: Kind,
    depth: f32,
}

const OFFSETS = array(
    vec2( 1.0,  1.0),
    vec2( 1.0, -1.0),
    vec2(-1.0,  1.0),
    vec2(-1.0, -1.0),
);
const NO_DEPTH: f32 = 0;
const WORLD: Kind = 0;
const AGENT: Kind = 1;
const OUTLINED: Kind = 2;
const BLACK = vec3<f32>(0);

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let in_front = in_front(in.uv);

    if in_front.kind == OUTLINED {
        return screen(in.uv, in_front);
    };

    let offset = 1.1 / vec2<f32>(textureDimensions(outline_texture));
    for (var i = 0; i < 4; i++) {
        let probe_uv = in.uv + OFFSETS[i] * offset;
        let in_front = in_front(probe_uv);

        if in_front.depth == NO_DEPTH || in_front.kind != OUTLINED {
            continue;
        }

        return outline_settings.outline_color;
    }

    return screen(in.uv, in_front);
}



fn load_depths(uv: vec2<f32>) -> Depths {
    let pos = vec2<i32>(uv * vec2<f32>(textureDimensions(world_depth).xy));

    return Depths(
        textureLoad(world_depth, pos, 0),
        textureLoad(agents_depth, pos, 0),
        textureLoad(outline_depth, pos, 0),
    );
}

fn in_front(uv: vec2<f32>) -> InFront {
    let depth = load_depths(uv);

    var in_front = InFront(
        WORLD,
        depth.world,
    );

    if depth.agent >= in_front.depth {
        in_front.kind = AGENT;
        in_front.depth = depth.agent;
    }

    if depth.outlined >= in_front.depth {
        in_front.kind = OUTLINED;
        in_front.depth = depth.outlined;
    }

    return in_front;
}

fn screen(uv: vec2<f32>, in_front: InFront) -> vec4<f32> {
    let screen = textureSample(screen_texture, screen_texture_sampler, uv);

    if is_visible(uv) == false {
        return screen * 0.1;
    }

    return screen;
}

fn is_visible(uv: vec2<f32>) -> bool {
    let visibility = textureSample(visibility_texture, visibility_texture_sampler, uv);

    if all(visibility.rgb == BLACK) {
        return false;
    }

    if visibility.a == 0. {
        return false;
    }

    return true;
}
