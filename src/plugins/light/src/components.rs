use bevy::{
	asset::Handle,
	ecs::{component::Component, entity::Entity},
	pbr::StandardMaterial,
};
use common::tools::{Intensity, IntensityChangePerSecond, Units};

#[derive(Debug, PartialEq, Clone)]
pub struct ResponsiveLightData {
	pub range: Units,
	pub light_on_material: Handle<StandardMaterial>,
	pub light_off_material: Handle<StandardMaterial>,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}

#[derive(Component, Debug, PartialEq, Clone)]
pub struct ResponsiveLight {
	pub model: Entity,
	pub light: Entity,
	pub data: ResponsiveLightData,
}

#[derive(Component)]
pub struct ResponsiveLightTrigger;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum ChangeLight {
	Increase(ResponsiveLight),
	Decrease(ResponsiveLight),
}
