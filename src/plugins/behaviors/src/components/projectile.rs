pub mod sub_type;

use super::{MovementConfig, MovementMode};
use crate::traits::{Caster, ProjectileBehavior, Spawner};
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_rapier3d::prelude::{Ccd, GravityScale, RigidBody};
use common::{
	blocker::Blocker,
	errors::Error,
	tools::UnitsPerSecond,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		prefab::{GetOrCreateAssets, Prefab},
	},
};
use interactions::components::is::{Fragile, Is};
use sub_type::SubType;

#[derive(Component, Debug, PartialEq)]
pub struct ProjectileContact {
	pub caster: Entity,
	pub spawner: Entity,
	pub range: f32,
	pub sub_type: SubType,
}

#[derive(Component, Debug, PartialEq)]
pub struct ProjectileProjection {
	pub sub_type: SubType,
}

impl Caster for ProjectileContact {
	fn caster(&self) -> Entity {
		self.caster
	}
}

impl Spawner for ProjectileContact {
	fn spawner(&self) -> Entity {
		self.spawner
	}
}

impl ProjectileBehavior for ProjectileContact {
	fn range(&self) -> f32 {
		self.range
	}
}

impl Prefab<()> for ProjectileContact {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity
			.try_insert((
				RigidBody::Dynamic,
				GravityScale(0.),
				Ccd::enabled(),
				Is::<Fragile>::interacting_with([Blocker::Physical, Blocker::Force]),
				MovementConfig::Constant {
					mode: MovementMode::Fast,
					speed: UnitsPerSecond::new(15.),
				},
			))
			.with_children(|parent| self.sub_type.spawn_contact(parent, &mut assets));

		Ok(())
	}
}

impl Prefab<()> for ProjectileProjection {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		entity.with_children(|parent| self.sub_type.spawn_projection(parent));

		Ok(())
	}
}
