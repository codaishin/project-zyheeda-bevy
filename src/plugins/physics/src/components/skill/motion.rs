use crate::{
	components::{
		anchor::Anchor,
		ground_target::GroundTarget,
		prevent_tunneling::PreventTunneling,
		set_velocity_forward::SetVelocityForward,
		skill::{
			CreatedFrom,
			PROJECTILE_CONTACT_RADIUS,
			PROJECTILE_RANGE,
			PROJECTILE_SPEED,
			Skill,
		},
		when_traveled::WhenTraveled,
	},
	observers::skill_prefab::ApplyMotionPrefab,
};
use bevy_rapier3d::prelude::*;
use common::{
	tools::Units,
	traits::handles_skill_physics::{SkillShape, ground_target::SphereAoE},
	zyheeda_commands::ZyheedaEntityCommands,
};

impl ApplyMotionPrefab for Skill {
	fn apply_motion_prefab(&self, entity: &mut ZyheedaEntityCommands) -> RigidBody {
		match &self.shape {
			SkillShape::SphereAoE(SphereAoE { max_range, .. }) => {
				entity.try_insert(GroundTarget {
					caster: self.caster,
					max_cast_range: *max_range,
					target: self.target,
				});

				RigidBody::Fixed
			}
			SkillShape::Projectile(..) => {
				entity.try_insert((
					GravityScale(0.),
					Ccd::enabled(),
					PreventTunneling {
						leading_edge: Units::from(PROJECTILE_CONTACT_RADIUS),
					},
					WhenTraveled::distance(PROJECTILE_RANGE).destroy(),
				));

				if self.created_from == CreatedFrom::Spawn {
					entity.try_insert((
						Anchor::attach_to(self.caster.0)
							.on(self.mount)
							.looking_at(self.target)
							.once(),
						SetVelocityForward(PROJECTILE_SPEED),
					));
				}

				RigidBody::Dynamic
			}
			SkillShape::Beam(..) => {
				entity.try_insert(
					Anchor::attach_to(self.caster.0)
						.on(self.mount)
						.looking_at(self.target)
						.always(),
				);

				RigidBody::Fixed
			}
			SkillShape::Shield(..) => {
				entity.try_insert(
					Anchor::attach_to(self.caster.0)
						.on(self.mount)
						.with_attached_rotation()
						.always(),
				);

				RigidBody::Fixed
			}
		}
	}
}
