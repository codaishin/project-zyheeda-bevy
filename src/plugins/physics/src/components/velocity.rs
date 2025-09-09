use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::traits::{
	handles_physics::Linear,
	register_derived_component::{DerivableFrom, InsertDerivedComponent},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[component(immutable)]
pub enum Motion {
	Linear(Vec3),
}

impl From<Linear> for Motion {
	fn from(Linear(velocity): Linear) -> Self {
		Self::Linear(velocity)
	}
}

impl DerivableFrom<'_, '_, Motion> for Velocity {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;
	type TParam = ();

	fn derive_from(_: Entity, physical_velocity: &Motion, _: &()) -> Self {
		match physical_velocity {
			Motion::Linear(velocity) => Velocity::linear(*velocity),
		}
	}
}
