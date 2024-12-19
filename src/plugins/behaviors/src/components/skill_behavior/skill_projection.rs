use super::SimplePrefab;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::{
		handles_behaviors::{ProjectionOffset, Shape},
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		prefab::{AfterInstantiation, GetOrCreateAssets, Prefab},
	},
};

#[derive(Component, Debug, Clone)]
pub struct SkillProjection {
	pub shape: Shape,
	pub offset: Option<ProjectionOffset>,
}

impl<TInteractions, TLifeCycles> Prefab<(TInteractions, TLifeCycles)> for SkillProjection
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
		let offset = match self.offset {
			Some(ProjectionOffset(offset)) => offset,
			_ => Vec3::ZERO,
		};

		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, offset)
	}
}
