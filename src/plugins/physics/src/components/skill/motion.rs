use crate::{
	components::{
		anchor::{Always, Anchor, Once},
		ground_target::GroundTarget,
		set_motion_forward::SetMotionForward,
		skill::{CreatedFrom, PROJECTILE_RANGE, PROJECTILE_SPEED, Skill},
		when_traveled::WhenTraveled,
	},
	observers::skill_prefab::ApplyMotionPrefab,
};
use bevy_rapier3d::prelude::*;
use common::{
	traits::handles_skill_physics::{SkillShape, ground_target::SphereAoE},
	zyheeda_commands::ZyheedaEntityCommands,
};

impl ApplyMotionPrefab for Skill {
	fn apply_motion_prefab(&self, entity: &mut ZyheedaEntityCommands) -> RigidBody {
		match &self.shape {
			SkillShape::SphereAoE(SphereAoE { max_range, .. }) => {
				entity.try_insert_if_new(GroundTarget {
					caster: self.caster,
					max_cast_range: *max_range,
					target: self.target,
				});

				RigidBody::Fixed
			}
			SkillShape::Projectile(..) => {
				entity.try_insert_if_new((
					GravityScale(0.),
					Ccd::enabled(),
					WhenTraveled::distance(PROJECTILE_RANGE).destroy(),
				));

				if self.created_from == CreatedFrom::Spawn {
					entity.try_insert_if_new((
						Anchor::<Once>::to_target(self.caster.0)
							.on_spawner(self.spawner)
							.with_target_rotation(),
						SetMotionForward(PROJECTILE_SPEED),
					));
				}

				RigidBody::Dynamic
			}
			SkillShape::Beam(..) | SkillShape::Shield(..) => {
				entity.try_insert_if_new(
					Anchor::<Always>::to_target(self.caster.0)
						.on_spawner(self.spawner)
						.with_target_rotation(),
				);

				RigidBody::Fixed
			}
		}
	}
}
