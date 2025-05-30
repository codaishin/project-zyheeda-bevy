use super::SimplePrefab;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{ProjectionOffset, Shape},
		prefab::Prefab,
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
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		let offset = match self.offset {
			Some(ProjectionOffset(offset)) => offset,
			_ => Vec3::ZERO,
		};

		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, offset)
	}
}
