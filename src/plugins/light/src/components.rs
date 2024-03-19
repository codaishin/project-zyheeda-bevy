use bevy::{
	asset::Handle,
	ecs::{component::Component, entity::Entity},
	pbr::StandardMaterial,
};

#[derive(Component)]
pub struct ResponsiveLight {
	pub range: f32,
	pub model: Entity,
	pub light: Entity,
	pub light_on_material: Handle<StandardMaterial>,
	pub light_off_material: Handle<StandardMaterial>,
}

#[derive(Component)]
pub struct ResponsiveLightTrigger;
