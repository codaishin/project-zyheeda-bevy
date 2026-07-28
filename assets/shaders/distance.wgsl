#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    forward_io::VertexOutput,
    mesh_view_bindings::view,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> range: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(in.world_position.xyz - view.world_position);

    return vec4(vec3(min(distance / range, 1.0)), 1.0);
}
