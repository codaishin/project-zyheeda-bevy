#import bevy_pbr::forward_io::VertexOutput
#import "shaders/helpers.wgsl"::fresnel

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> fill_color: vec4<f32>;
@group(2) @binding(2) var<uniform> shine_color: vec4<f32>;
@group(2) @binding(3) var material_color_texture: texture_2d<f32>;
@group(2) @binding(4) var material_color_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let fresnel = fresnel(mesh, 1.);
    var color = material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv);
    color = mix(color, fill_color, 1. - color.a);

    return mix(color, shine_color, fresnel);
}
