use super::SimplePrefab;
use bevy::prelude::*;
use common::{
	errors::Error,
	traits::{
		handles_interactions::HandlesInteractions,
		handles_skill_behaviors::{Projection, ProjectionOffset, Shape},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, Clone, Serialize, Deserialize)]
pub struct SkillProjection {
	pub shape: Shape,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub offset: Option<ProjectionOffset>,
}

impl From<Projection> for SkillProjection {
	fn from(Projection { shape, offset }: Projection) -> Self {
		Self { shape, offset }
	}
}

impl<TInteractions> Prefab<TInteractions> for SkillProjection
where
	TInteractions: HandlesInteractions,
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

		self.shape.prefab::<TInteractions>(entity, offset)
	}
}
