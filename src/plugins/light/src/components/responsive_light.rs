use std::ops::Deref;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, CollidingEntities, Sensor};
use common::{
	errors::Error,
	tools::{Intensity, IntensityChangePerSecond, Units},
	traits::{
		handles_lights::Responsive,
		prefab::{GetOrCreateAssets, Prefab},
	},
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

impl Prefab<()> for ResponsiveLight {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity.try_insert((
			TransformBundle::default(),
			Collider::ball(*self.range.deref()),
			Sensor,
			ActiveEvents::COLLISION_EVENTS,
			CollidingEntities::default(),
		));

		Ok(())
	}
}
