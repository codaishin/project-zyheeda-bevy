use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::traits::{
	handles_physics::Linear,
	register_derived_component::{DerivableFrom, InsertDerivedComponent},
};

#[derive(Component, Debug, PartialEq)]
pub enum Motion {
	Linear(Linear),
}

impl From<Linear> for Motion {
	fn from(velocity: Linear) -> Self {
		Self::Linear(velocity)
	}
}

impl DerivableFrom<'_, '_, Motion> for Velocity {
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;
	type TParam = ();

	fn derive_from(physical_velocity: &Motion, _: &()) -> Self {
		match physical_velocity {
			Motion::Linear(Linear(velocity)) => Velocity::linear(*velocity),
		}
	}
}
