use super::{Integrity, Motion, Shape};
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
	},
};

#[derive(Component, Debug, Clone)]
pub struct SkillContact {
	pub shape: Shape,
	pub integrity: Integrity,
	pub motion: Motion,
}

impl<TInteractions, TLifeCycles> Prefab<(TInteractions, TLifeCycles)> for SkillContact
where
	TInteractions: HandlesInteractions,
	TLifeCycles: HandlesDestruction,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error>
	where
		TAfterInstantiation: AfterInstantiation,
	{
		self.shape.prefab(entity, Vec3::ZERO)?;
		self.motion.prefab::<TLifeCycles>(entity)?;
		self.integrity.prefab::<TInteractions>(entity)?;

		Ok(())
	}
}
