use crate::{
	behaviors::SkillCaster,
	components::SkillTarget,
	skills::lifetime_definition::LifeTimeDefinition,
	traits::skill_builder::{BuildContact, BuildProjection, SkillLifetime},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use common::{
	components::AssetModel,
	tools::Units,
	traits::{
		clamp_zero_positive::ClampZeroPositive,
		handles_skill_behaviors::{
			HandlesSkillBehaviors,
			Integrity,
			Motion,
			ProjectionOffset,
			Shape,
			Spawner,
		},
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SpawnShield;

impl BuildContact for SpawnShield {
	fn build_contact<TSkillBehaviors>(
		&self,
		caster: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) -> TSkillBehaviors::TSkillContact
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		let SkillCaster(caster) = *caster;

		TSkillBehaviors::skill_contact(
			Shape::Custom {
				model: AssetModel::path("models/shield.glb"),
				collider: Collider::cuboid(0.5, 0.5, 0.05),
				scale: Vec3::splat(1.),
			},
			Integrity::Solid,
			Motion::HeldBy { caster },
		)
	}
}

impl BuildProjection for SpawnShield {
	fn build_projection<TSkillBehaviors>(
		&self,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) -> TSkillBehaviors::TSkillProjection
	where
		TSkillBehaviors: HandlesSkillBehaviors,
	{
		let radius = 1.;
		let offset = Vec3::new(0., 0., radius);

		TSkillBehaviors::skill_projection(
			Shape::Sphere {
				radius: Units::new(radius),
				hollow_collider: false,
			},
			Some(ProjectionOffset(offset)),
		)
	}
}

impl SkillLifetime for SpawnShield {
	fn lifetime(&self) -> LifeTimeDefinition {
		LifeTimeDefinition::UntilStopped
	}
}
