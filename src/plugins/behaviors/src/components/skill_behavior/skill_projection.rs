use super::{Offset, Shape};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
};

#[derive(Component, Debug, Clone)]
pub struct SkillProjection {
	pub shape: Shape,
	pub offset: Option<Offset>,
}

impl Prefab<()> for SkillProjection {
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation,
	{
		let offset = match self.offset {
			Some(Offset(offset)) => offset,
			_ => Vec3::ZERO,
		};

		self.shape.prefab(entity, offset)
	}
}
