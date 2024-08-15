use super::{GetStaticSkillBehavior, SkillBundleConfig, SpawnSkill};
use crate::skills::{
	SkillBehavior,
	SkillBehaviors,
	SkillCaster,
	SkillSpawnAndExecute,
	SkillSpawner,
	Target,
};
use behaviors::components::{gravity_well::GravityWell, ground_target::GroundTarget};
use bevy::{ecs::bundle::Bundle, prelude::Transform, utils::default};
use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};

impl SkillBundleConfig for GravityWell {
	const STOPPABLE: bool = false;

	fn new_skill_bundle(caster: &SkillCaster, _: &SkillSpawner, target: &Target) -> impl Bundle {
		(
			GravityWell,
			GroundTarget {
				caster: Transform::from(caster.1),
				target_ray: target.ray,
				max_range: Units::new(10.),
			},
		)
	}
}

impl GetStaticSkillBehavior for GravityWell {
	fn behavior() -> SkillBehavior {
		SkillBehavior::OnActive(SkillBehaviors {
			projection: SkillSpawnAndExecute {
				spawn: GravityWell::spawn_skill,
				..default()
			},
			..default()
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::entity::Entity,
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::test_tools::utils::TryCast;

	#[test]
	fn new_skill_bundle() {
		assert_eq!(
			Ok(&(
				GravityWell,
				GroundTarget {
					caster: Transform::from_xyz(1., 2., 3.),
					target_ray: Ray3d::new(Vec3::new(11., 12., 14.), Vec3::new(4., 2., 0.)),
					max_range: Units::new(10.)
				},
			)),
			GravityWell::new_skill_bundle(
				&SkillCaster(Entity::from_raw(0), GlobalTransform::from_xyz(1., 2., 3.)),
				&SkillSpawner(Entity::from_raw(0), GlobalTransform::from_xyz(4., 5., 6.)),
				&Target {
					ray: Ray3d::new(Vec3::new(11., 12., 14.), Vec3::new(4., 2., 0.)),
					collision_info: None,
				},
			)
			.try_cast::<(GravityWell, GroundTarget)>()
		)
	}
}
