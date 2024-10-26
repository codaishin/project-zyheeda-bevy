/// Adapted from fresnel example in https://github.com/rust-adventure/bevy-examples
fn fresnel(
    mesh_world_normal: vec3<f32>,
    mesh_world_position: vec4<f32>,
    view_world_position: vec3<f32>
) -> f32 {
    let normal = normalize(mesh_world_normal);
    let view_vector = normalize(view_world_position - mesh_world_position.xyz);
    let normalized_angle = dot(normal, view_vector);
    return clamp(1.0 - normalized_angle, 0.0, 1.0);
}

struct DistortParams {
    falloff: f32,
    intensity: f32,
}

fn distort(value: f32, params: DistortParams) -> f32 {
    return pow(value, params.falloff) * params.intensity;
}
