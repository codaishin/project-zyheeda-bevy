pub mod sub_type;

use super::{Force, MovementConfig, MovementMode};
use crate::traits::ProjectileBehavior;
use bevy::{
	self,
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::Dir3,
	prelude::Component,
	utils::default,
};
use bevy_rapier3d::prelude::RigidBody;
use common::{
	components::{PhysicalEntity, Wall},
	errors::Error,
	test_tools::utils::ApproxEqual,
	tools::UnitsPerSecond,
	traits::clamp_zero_positive::ClampZeroPositive,
};
use interactions::{components::is::Is, traits::ConcatBlockers};
use prefabs::traits::{GetOrCreateAssets, Instantiate};
use sub_type::SubType;

#[derive(Component, Debug, PartialEq)]
pub struct Projectile {
	pub direction: Dir3,
	pub range: f32,
	pub sub_type: SubType,
}

impl Default for Projectile {
	fn default() -> Self {
		Self {
			direction: Dir3::NEG_Z,
			range: default(),
			sub_type: default(),
		}
	}
}

impl ApproxEqual<f32> for Projectile {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.direction
			.as_vec3()
			.approx_equal(&other.direction.as_vec3(), tolerance)
			&& self.range == other.range
			&& self.sub_type == other.sub_type
	}
}

impl ProjectileBehavior for Projectile {
	fn direction(&self) -> Dir3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}

impl Instantiate for Projectile {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		on.try_insert((
			RigidBody::Fixed,
			Is::fragile()
				.blocked_by::<PhysicalEntity>()
				.and::<Wall>()
				.and::<Force>(),
			MovementConfig::Constant {
				mode: MovementMode::Fast,
				speed: UnitsPerSecond::new(15.),
			},
		))
		.with_children(|parent| self.sub_type.spawn(parent, &mut assets));

		Ok(())
	}
}
