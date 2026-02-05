use crate::components::motion::Motion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{KinematicCharacterController, *};
use common::{
	tools::speed::Speed,
	traits::{accessors::get::TryApplyOn, handles_physics::LinearMotion},
	zyheeda_commands::ZyheedaCommands,
};
use std::time::Duration;

impl Motion {
	pub(crate) fn apply(
		delta: In<Duration>,
		motions: Query<(Entity, &Self, &Transform)>,
		mut commands: ZyheedaCommands,
		mut characters: Query<&mut KinematicCharacterController>,
	) {
		let delta = delta.as_secs_f32();

		for (entity, motion, transform) in &motions {
			let (dir, speed) = match motion {
				Motion::Ongoing(LinearMotion::Direction { speed, direction }) => {
					(**direction, *speed)
				}
				Motion::Ongoing(LinearMotion::ToTarget { speed, target }) => {
					(direction_to(*target, transform), *speed)
				}
				Motion::Ongoing(LinearMotion::Stop) | Motion::Done(..) => (Vec3::ZERO, Speed::ZERO),
			};

			match characters.get_mut(entity) {
				Ok(mut character) => {
					character.translation = Some(dir * *speed * delta);
				}
				Err(_) => {
					commands.try_apply_on(&entity, |mut e| {
						e.try_insert(Velocity::linear(dir * *speed));
					});
				}
			}
		}
	}
}

fn direction_to(target: Vec3, transform: &Transform) -> Vec3 {
	let direction = target - transform.translation;
	direction.try_normalize().unwrap_or_default()
}
