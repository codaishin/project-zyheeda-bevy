#import bevy_pbr::forward_io::Vertex
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import "shaders/helpers.wgsl"::fresnel

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> fill_color: vec4<f32>;
@group(2) @binding(2) var<uniform> shine_color: vec4<f32>;
@group(2) @binding(3) var material_color_texture: texture_2d<f32>;
@group(2) @binding(4) var material_color_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let fresnel = fresnel(mesh.world_normal, mesh.world_position, view.world_position);
    var color = material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv);
    color = mix(color, fill_color, 1. - color.a);

    return mix(color, shine_color, fresnel);
}
