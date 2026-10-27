import bevy_pbr::render::{forward_io::VertexOutput, mesh_view_bindings::view};

/// Adapted from fresnel example in https://github.com/rust-adventure/bevy-examples
fn fresnel(mesh: VertexOutput, power: f32) -> f32 {
    let normal = normalize(mesh.world_normal);
    let view_vector = normalize(view.world_position - mesh.world_position.xyz);
    let angle = clamp(dot(normal, view_vector), 0., 1.);
    return pow(1.0 - angle, power);
}
