use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use common::{
	components::{asset_model::AssetModel, is_blocker::Blocker},
	tools::Units,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_skill_behaviors::{
			Contact,
			ContactShape,
			HandlesSkillBehaviors,
			Motion,
			Projection,
			ProjectionOffset,
			ProjectionShape,
			SkillEntities,
			SkillSpawner,
		},
	},
	zyheeda_commands::ZyheedaCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield;

impl SpawnShape for SpawnShield {
	fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		_: SkillSpawner,
		_: &SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let SkillCaster(caster) = *caster;
		let radius = 1.;
		let offset = Vec3::new(0., 0., -radius);

		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: ContactShape::Custom {
					model: AssetModel::path("models/shield.glb").flipped_on("Shield"),
					collider: Collider::cuboid(0.5, 0.5, 0.05),
					scale: Vec3::splat(1.),
					destroyed_by: Blocker::none(),
				},
				motion: Motion::HeldBy {
					caster,
					spawner: SkillSpawner::Neutral,
				},
			},
			Projection {
				shape: ProjectionShape::Sphere {
					radius: Units::new(radius),
				},
				offset: Some(ProjectionOffset(offset)),
			},
		)
	}
}

impl SkillLifetime for SpawnShield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
