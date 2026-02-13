use crate::components::character_motion::{ApplyCharacterMotion, IsInMotion};
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
		motions: Query<(Entity, &Self, &Transform), With<IsInMotion>>,
	) {
		for (entity, apply, transform) in motions {
			if !is_done(&apply.motion, transform, delta) {
				continue;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(apply.is_done());
				e.try_remove::<IsInMotion>();
			});
		}
	}

	fn is_done(&self) -> Self {
		Self {
			motion: self.motion,
			is_done: true,
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
	#![allow(clippy::unwrap_used)]
	use crate::components::character_motion::IsInMotion;

	use super::*;
	use common::{
		tools::{UnitsPerSecond, speed::Speed},
		traits::handles_physics::CharacterMotion,
	};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				(move || delta).pipe(ApplyCharacterMotion::set_done),
				IsChanged::<ApplyCharacterMotion>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_done_when_target_is_stop() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::default(),
				ApplyCharacterMotion::from(CharacterMotion::Stop),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(true), None),
			(
				app.world()
					.entity(entity)
					.get::<ApplyCharacterMotion>()
					.map(|m| m.is_done),
				app.world().entity(entity).get::<IsInMotion>()
			)
		);
	}

	#[test]
	fn set_done_when_target_is_translation() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1., 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(true), None),
			(
				app.world()
					.entity(entity)
					.get::<ApplyCharacterMotion>()
					.map(|m| m.is_done),
				app.world().entity(entity).get::<IsInMotion>()
			)
		);
	}

	#[test]
	fn do_not_set_done_when_target_is_not_translation() {
		let mut app = setup(Duration::default());
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(10., 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(false), Some(&IsInMotion)),
			(
				app.world()
					.entity(entity)
					.get::<ApplyCharacterMotion>()
					.map(|m| m.is_done),
				app.world().entity(entity).get::<IsInMotion>()
			)
		);
	}

	#[test]
	fn set_done_when_target_one_delta_away_from_target() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1.099, 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(true), None),
			(
				app.world()
					.entity(entity)
					.get::<ApplyCharacterMotion>()
					.map(|m| m.is_done),
				app.world().entity(entity).get::<IsInMotion>()
			)
		);
	}

	#[test]
	fn set_done_when_target_one_delta_away_from_target_accounting_for_speed() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(2.)),
					target: Vec3::new(1.199, 2., 3.),
				}),
			))
			.id();

		app.update();

		assert_eq!(
			(Some(true), None),
			(
				app.world()
					.entity(entity)
					.get::<ApplyCharacterMotion>()
					.map(|m| m.is_done),
				app.world().entity(entity).get::<IsInMotion>()
			)
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::default(),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::ZERO,
				}),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ApplyCharacterMotion>>(),
		);
	}

	#[test]
	fn act_again_when_is_in_motion_present() {
		let mut app = setup(Duration::from_millis(100));
		let entity = app
			.world_mut()
			.spawn((
				Transform::default(),
				ApplyCharacterMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::ZERO,
				}),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).insert(IsInMotion);
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world()
				.entity(entity)
				.get::<IsChanged<ApplyCharacterMotion>>(),
		);
	}
}
