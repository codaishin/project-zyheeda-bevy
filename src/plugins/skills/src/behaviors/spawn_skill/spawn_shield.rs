use crate::{
	behaviors::SkillCaster,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{SkillLifetime, SpawnShape},
};
use bevy::prelude::*;
use common::{
	components::asset_model::AssetModel,
	tools::Units,
	traits::{
		handles_physics::colliders::{Blocker, Shape},
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
			SkillTarget,
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
		caster: SkillCaster,
		_: SkillSpawner,
		_: SkillTarget,
	) -> SkillEntities
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		let radius = 1.;
		let offset = Vec3::new(0., 0., -radius);

		TSkillBehaviors::spawn_skill(
			commands,
			Contact {
				shape: ContactShape::Custom {
					model: AssetModel::path("models/shield.glb").flipped_on("Shield"),
					collider: Shape::Cuboid {
						half_x: Units::from(0.5),
						half_y: Units::from(0.5),
						half_z: Units::from(0.05),
					},
					model_scale: Vec3::splat(1.),
					destroyed_by: Blocker::none(),
				},
				motion: Motion::HeldBy {
					caster,
					spawner: SkillSpawner::Neutral,
				},
			},
			Projection {
				shape: ProjectionShape::Sphere {
					radius: Units::from(radius),
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
