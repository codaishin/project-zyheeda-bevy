use bevy::ecs::component::Component;

#[derive(Component)]
pub(crate) struct Destroy;

#[derive(Component)]
pub struct DealsDamage(pub i16);
