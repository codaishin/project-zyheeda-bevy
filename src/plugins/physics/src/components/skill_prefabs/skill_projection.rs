use super::SkillPrefab;
use crate::components::skill_prefabs::FaultyColliderShape;
use bevy::prelude::*;
use common::traits::{
	handles_skill_behaviors::{Projection, ProjectionOffset, ProjectionShape},
	load_asset::LoadAsset,
	prefab::{Prefab, PrefabEntityCommands},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, Clone, Serialize, Deserialize)]
pub struct SkillProjection {
	pub shape: ProjectionShape,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub offset: Option<ProjectionOffset>,
}

impl From<Projection> for SkillProjection {
	fn from(Projection { shape, offset }: Projection) -> Self {
		Self { shape, offset }
	}
}

impl Prefab<()> for SkillProjection {
	type TError = FaultyColliderShape;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		_: &mut impl LoadAsset,
	) -> Result<(), FaultyColliderShape> {
		let offset = match self.offset {
			Some(ProjectionOffset(offset)) => offset,
			_ => Vec3::ZERO,
		};

		self.shape.prefab(entity, offset)
	}
}
