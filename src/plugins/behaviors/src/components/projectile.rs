use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::prefab::{GetOrCreateAssets, Prefab},
};

#[derive(Component, Debug, PartialEq)]
pub struct ProjectileProjection;

impl Prefab<()> for ProjectileProjection {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		_: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		Ok(())
	}
}
