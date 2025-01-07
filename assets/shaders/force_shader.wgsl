#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view
#import "shaders/helpers.wgsl"::fresnel
#import "shaders/helpers.wgsl"::distort
#import "shaders/helpers.wgsl"::DistortParams

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var distort_params: DistortParams;
    distort_params.falloff = 6.;
    distort_params.intensity = 2.;

    var fresnel = fresnel(mesh.world_normal, mesh.world_position, view.world_position);
    fresnel = distort(fresnel, distort_params);

    return vec4(material_color.rgb, material_color.a * fresnel);
}
