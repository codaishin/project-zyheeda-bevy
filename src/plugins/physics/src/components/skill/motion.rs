use crate::{
	components::{
		fix_points::{Always, Anchor, Once},
		ground_target::GroundTarget,
		set_motion_forward::SetMotionForward,
		skill::{CreatedFrom, Skill},
		when_traveled::WhenTraveled,
	},
	observers::skill_prefab::ApplyMotionPrefab,
};
use bevy_rapier3d::prelude::*;
use common::{traits::handles_skill_physics::Motion, zyheeda_commands::ZyheedaEntityCommands};

impl ApplyMotionPrefab for Skill {
	fn apply_motion_prefab(&self, entity: &mut ZyheedaEntityCommands) -> RigidBody {
		match self.contact.motion {
			Motion::HeldBy { caster, spawner } => {
				entity.try_insert_if_new(
					Anchor::<Always>::to_target(caster.0)
						.on_spawner(spawner)
						.with_target_rotation(),
				);

				RigidBody::Fixed
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target,
			} => {
				entity.try_insert_if_new(GroundTarget {
					caster,
					max_cast_range,
					target,
				});

				RigidBody::Fixed
			}
			Motion::Projectile {
				caster,
				spawner,
				speed,
				range,
			} => {
				entity.try_insert_if_new((
					GravityScale(0.),
					Ccd::enabled(),
					WhenTraveled::distance(range).destroy(),
				));

				if self.created_from == CreatedFrom::Spawn {
					entity.try_insert_if_new((
						Anchor::<Once>::to_target(caster.0)
							.on_spawner(spawner)
							.with_target_rotation(),
						SetMotionForward(speed),
					));
				}

				RigidBody::Dynamic
			}
		}
	}
}
