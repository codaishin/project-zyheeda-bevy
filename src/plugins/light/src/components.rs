use bevy::{
	asset::Handle,
	ecs::{component::Component, entity::Entity},
	pbr::StandardMaterial,
};
use common::{
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::handles_lights::Responsive,
};

#[derive(Component, Debug, PartialEq, Clone)]
pub struct ResponsiveLight {
	pub model: Entity,
	pub light: Entity,
	pub range: Units,
	pub light_on_material: Handle<StandardMaterial>,
	pub light_off_material: Handle<StandardMaterial>,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}

impl From<Responsive> for ResponsiveLight {
	fn from(data: Responsive) -> Self {
		ResponsiveLight {
			model: data.model,
			light: data.light,
			range: data.range,
			light_on_material: data.light_on_material,
			light_off_material: data.light_off_material,
			max: data.max,
			change: data.change,
		}
	}
}

#[derive(Component)]
pub struct ResponsiveLightTrigger;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum ChangeLight {
	Increase(ResponsiveLight),
	Decrease(ResponsiveLight),
}
