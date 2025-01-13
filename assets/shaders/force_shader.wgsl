#import bevy_pbr::forward_io::VertexOutput
#import "shaders/helpers.wgsl"::fresnel

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let fresnel = fresnel(mesh, 5.);

    return vec4(material_color.rgb, material_color.a * fresnel);
}
