use crate::skills::{
	behaviors::{SkillContact, SkillLifetime, SkillProjection},
	lifetime_definition::LifeTimeDefinition,
};
use bevy::prelude::*;
use common::{
	components::asset_model::AssetModel,
	tools::Units,
	traits::{
		handles_physics::colliders::{Blocker, Shape},
		handles_skill_physics::{
			Contact,
			ContactShape,
			Motion,
			Projection,
			ProjectionOffset,
			ProjectionShape,
			SkillCaster,
			SkillSpawner,
			SkillTarget,
		},
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Shield;

impl Shield {
	const RADIUS: f32 = 1.;
	const PROJECTION_OFFSET: Vec3 = Vec3::new(0., 0., -Self::RADIUS);
}

impl SkillContact for Shield {
	fn skill_contact(&self, caster: SkillCaster, _: SkillSpawner, _: SkillTarget) -> Contact {
		Contact {
			shape: ContactShape::Custom {
				model: AssetModel::path("models/shield.glb").flipped_on("Shield"),
				collider: Shape::Cuboid {
					half_x: Units::from(0.5),
					half_y: Units::from(0.5),
					half_z: Units::from(0.05),
				},
				model_scale: Vec3::ONE,
				destroyed_by: Blocker::none(),
			},
			motion: Motion::HeldBy {
				caster,
				spawner: SkillSpawner::Neutral,
			},
		}
	}
}

impl SkillProjection for Shield {
	fn skill_projection(&self) -> Projection {
		Projection {
			shape: ProjectionShape::Sphere {
				radius: Units::from(Self::RADIUS),
			},
			offset: Some(ProjectionOffset(Self::PROJECTION_OFFSET)),
		}
	}
}

impl SkillLifetime for Shield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
