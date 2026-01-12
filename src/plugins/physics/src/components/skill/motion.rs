use crate::components::{
	fix_points::{Always, Anchor, Once},
	ground_target::GroundTarget,
	set_motion_forward::SetMotionForward,
	skill::{CreatedFrom, Skill},
	when_traveled::WhenTraveled,
};
use bevy_rapier3d::prelude::*;
use common::{traits::handles_skill_physics::Motion, zyheeda_commands::ZyheedaEntityCommands};

impl Skill {
	pub(crate) fn motion(&self, entity: &mut ZyheedaEntityCommands) {
		match self.contact.motion {
			Motion::HeldBy { caster, spawner } => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					Anchor::<Always>::to_target(caster.0)
						.on_spawner(spawner)
						.with_target_rotation(),
				));
			}
			Motion::Stationary {
				caster,
				max_cast_range,
				target,
			} => {
				entity.try_insert_if_new((
					RigidBody::Fixed,
					GroundTarget {
						caster,
						max_cast_range,
						target,
					},
				));
			}
			Motion::Projectile {
				caster,
				spawner,
				speed,
				range,
			} => {
				entity.try_insert_if_new((
					RigidBody::Dynamic,
					GravityScale(0.),
					Ccd::enabled(),
					WhenTraveled::distance(range).destroy(),
				));

				if self.created_from == CreatedFrom::Save {
					return;
				}

				entity.try_insert_if_new((
					Anchor::<Once>::to_target(caster.0)
						.on_spawner(spawner)
						.with_target_rotation(),
					SetMotionForward(speed),
				));
			}
		}
	}
}
