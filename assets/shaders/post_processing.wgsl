#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding( 0) var world_depth: texture_depth_2d;
@group(0) @binding( 1) var world_depth_sampler: sampler_comparison;
@group(0) @binding( 2) var agents_depth: texture_depth_2d;
@group(0) @binding( 3) var agents_depth_sampler: sampler_comparison;
@group(0) @binding( 4) var outline_depth: texture_depth_2d;
@group(0) @binding( 5) var outline_depth_sampler: sampler_comparison;
@group(0) @binding( 6) var screen_texture: texture_2d<f32>;
@group(0) @binding( 7) var screen_texture_sampler: sampler;
@group(0) @binding( 8) var visibility_texture: texture_2d<f32>;
@group(0) @binding( 9) var visibility_texture_sampler: sampler;
@group(0) @binding(10) var<uniform> settings: PostProcessSettings;

alias Kind = u32;
alias Pixel = f32;

struct PostProcessSettings {
    outline_color: vec4<f32>,
    outline_width: Pixel,
    see_through_color: vec4<f32>,
    dark_region_light_factor: f32,
}

struct Depths {
    world: f32,
    agent: f32,
    outlined: f32,
}

struct ScreenInfo {
    order: array<LayerInfo, 3>,
}

struct LayerInfo {
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
    let info = info(in.uv);

    if info.order[0].kind == OUTLINED {
        return screen(in.uv, info);
    };

    let offset = settings.outline_width / vec2<f32>(textureDimensions(outline_depth));
    for (var i = 0; i < 4; i++) {
        let probe_uv = in.uv + OFFSETS[i] * offset;
        let info = info(probe_uv);

        if info.order[0].depth == NO_DEPTH || info.order[0].kind != OUTLINED {
            continue;
        }

        return settings.outline_color;
    }

    return screen(in.uv, info);
}

fn load_depths(uv: vec2<f32>) -> Depths {
    let pos = vec2<i32>(uv * vec2<f32>(textureDimensions(world_depth).xy));

    return Depths(
        textureLoad(world_depth, pos, 0),
        textureLoad(agents_depth, pos, 0),
        textureLoad(outline_depth, pos, 0),
    );
}

fn info(uv: vec2<f32>) -> ScreenInfo {
    let depth = load_depths(uv);
    var order = array(
        LayerInfo(WORLD, depth.world),
        LayerInfo(AGENT, depth.agent),
        LayerInfo(OUTLINED, depth.outlined),
    );

    if order[1].depth >= order[0].depth {
        let tmp = order[0];
        order[0] = order[1];
        order[1] = tmp;
    }

    if order[2].depth >= order[1].depth {
        let tmp = order[1];
        order[1] = order[2];
        order[2] = tmp;
    }

    if order[1].depth >= order[0].depth {
        let tmp = order[0];
        order[0] = order[1];
        order[1] = tmp;
    }

    return ScreenInfo(order);
}

fn screen(uv: vec2<f32>, info: ScreenInfo) -> vec4<f32> {
    if info.order[1].kind == AGENT && info.order[1].depth != NO_DEPTH {
        return settings.see_through_color;
    }

    let visibility = textureSample(visibility_texture, visibility_texture_sampler, uv);
    let screen = textureSample(screen_texture, screen_texture_sampler, uv);

    if all(visibility.rgb == BLACK) || visibility.a == 0 {
        return screen * settings.dark_region_light_factor;
    }

    return screen;
}
