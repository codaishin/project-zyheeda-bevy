use super::SimplePrefab;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	errors::Error,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Integrity, Motion, Shape},
		prefab::Prefab,
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
	fn insert_prefab_components(&self, entity: &mut EntityCommands) -> Result<(), Error> {
		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, Vec3::ZERO)?;
		self.motion
			.prefab::<TInteractions, TLifeCycles>(entity, ())?;
		self.integrity
			.prefab::<TInteractions, TLifeCycles>(entity, ())
	}
}
