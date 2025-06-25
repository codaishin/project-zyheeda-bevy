use super::SimplePrefab;
use bevy::prelude::*;
use common::{
	errors::Error,
	impl_savable_self_non_priority,
	traits::{
		handles_destruction::HandlesDestruction,
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Projection, ProjectionOffset, Shape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SkillProjection {
	pub shape: Shape,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub offset: Option<ProjectionOffset>,
}

impl_savable_self_non_priority!(SkillProjection);

impl From<Projection> for SkillProjection {
	fn from(Projection { shape, offset }: Projection) -> Self {
		Self { shape, offset }
	}
}

impl<TInteractions, TLifeCycles> Prefab<(TInteractions, TLifeCycles)> for SkillProjection
where
	TInteractions: HandlesInteractions,
	TLifeCycles: HandlesDestruction,
{
	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), Error> {
		let offset = match self.offset {
			Some(ProjectionOffset(offset)) => offset,
			_ => Vec3::ZERO,
		};

		self.shape
			.prefab::<TInteractions, TLifeCycles>(entity, offset)
	}
}
