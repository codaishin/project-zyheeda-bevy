use crate::components::character_motion::ApplyCharacterMotion;
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::CharacterMotion},
	zyheeda_commands::ZyheedaCommands,
};
use std::time::Duration;

impl ApplyCharacterMotion {
	pub(crate) fn set_done(
		In(delta): In<Duration>,
		mut commands: ZyheedaCommands,
		motions: Query<(Entity, &Self, &Transform)>,
	) {
		for (entity, motion, transform) in &motions {
			let ApplyCharacterMotion::Ongoing(motion) = motion else {
				continue;
			};

			if !is_done(motion, transform, delta) {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Self::Done(*motion));
			});
		}
	}
}

fn is_done(motion: &CharacterMotion, transform: &Transform, delta: Duration) -> bool {
	let (speed, target) = match motion {
		CharacterMotion::Direction { .. } => return false,
		CharacterMotion::Stop => return true,
		CharacterMotion::ToTarget { speed, target } => (speed, target),
	};

	if target == &transform.translation {
		return true;
	}

	let distance_to_target = (target - transform.translation).length();
	let distance_traveled_per_frame = delta.as_secs_f32() * **speed;

	distance_to_target < distance_traveled_per_frame
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::{UnitsPerSecond, speed::Speed},
		traits::handles_physics::CharacterMotion,
	};
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, (move || delta).pipe(ApplyCharacterMotion::set_done));

		app
	}

	#[test]
	fn set_done_when_target_is_stop() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::default(),
				ApplyCharacterMotion::Ongoing(CharacterMotion::Stop),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyCharacterMotion::Done(CharacterMotion::Stop)),
			app.world().entity(entity).get::<ApplyCharacterMotion>(),
		);
	}

	#[test]
	fn set_done_when_target_is_translation() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1., 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyCharacterMotion::Done(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::new(1., 2., 3.),
			})),
			app.world().entity(entity).get::<ApplyCharacterMotion>(),
		);
	}

	#[test]
	fn do_not_set_done_when_target_is_not_translation() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(10., 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::new(10., 2., 3.),
			})),
			app.world().entity(entity).get::<ApplyCharacterMotion>(),
		);
	}

	#[test]
	fn set_done_when_target_one_delta_away_from_target() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1.099, 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyCharacterMotion::Done(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::new(1.099, 2., 3.),
			})),
			app.world().entity(entity).get::<ApplyCharacterMotion>(),
		);
	}

	#[test]
	fn set_done_when_target_one_delta_away_from_target_accounting_for_speed() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(2.)),
					target: Vec3::new(1.199, 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&ApplyCharacterMotion::Done(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(2.)),
				target: Vec3::new(1.199, 2., 3.),
			})),
			app.world().entity(entity).get::<ApplyCharacterMotion>(),
		);
	}
}
