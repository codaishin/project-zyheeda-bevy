use crate::components::config::{Config, SpeedIndex};
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::{
		accessors::get::{TryApplyOn, View},
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> UpdateSpeed for T where T: Component + From<CharacterMotion> + View<CharacterMotion> {}

pub(crate) trait UpdateSpeed:
	Component + From<CharacterMotion> + View<CharacterMotion> + Sized
{
	fn update_speed(
		mut commands: ZyheedaCommands,
		motions: Query<(Entity, &Self, &Config, &SpeedIndex), Changed<SpeedIndex>>,
	) {
		for (entity, motion, config, speed_index) in motions {
			let new_motion = match motion.view() {
				CharacterMotion::Done => continue,
				CharacterMotion::Direction { direction, .. } => CharacterMotion::Direction {
					speed: Speed(config[*speed_index]),
					direction,
				},
				CharacterMotion::ToTarget { target, .. } => CharacterMotion::ToTarget {
					speed: Speed(config[*speed_index]),
					target,
				},
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(Self::from(new_motion));
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::UnitsPerSecond,
		traits::handles_movement::{MovementSpeed, SpeedToggle},
	};
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Motion(CharacterMotion);

	impl From<CharacterMotion> for _Motion {
		fn from(value: CharacterMotion) -> Self {
			Self(value)
		}
	}

	impl View<CharacterMotion> for _Motion {
		fn view(&self) -> CharacterMotion {
			self.0
		}
	}

	const SLOW: UnitsPerSecond = UnitsPerSecond::from_u8(1);
	const FAST: UnitsPerSecond = UnitsPerSecond::from_u8(2);

	fn with_speed(motion: CharacterMotion, new_speed: Speed) -> CharacterMotion {
		match motion {
			CharacterMotion::Direction { direction, .. } => CharacterMotion::Direction {
				speed: new_speed,
				direction,
			},
			CharacterMotion::ToTarget { target, .. } => CharacterMotion::ToTarget {
				speed: new_speed,
				target,
			},
			CharacterMotion::Done => CharacterMotion::Done,
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(_Motion::update_speed, IsChanged::<_Motion>::detect).chain(),
		);

		app
	}

	#[test_case(CharacterMotion::Direction { direction: Dir3::Z, speed: Speed(SLOW) }; "direction")]
	#[test_case(CharacterMotion::ToTarget { target: Vec3::Z, speed: Speed(SLOW) }; "target")]
	#[test_case(CharacterMotion::Done; "done")]
	fn update_speed(motion: CharacterMotion) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(motion),
				Config {
					speed: MovementSpeed::Variable([SLOW, FAST]),
					..default()
				},
				SpeedIndex(SpeedToggle::Right),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion(with_speed(motion, Speed(FAST)))),
			app.world().entity(entity).get::<_Motion>(),
		)
	}

	#[test]
	fn keep_done_unchanged() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(CharacterMotion::Done),
				Config {
					speed: MovementSpeed::Variable([SLOW, FAST]),
					..default()
				},
				SpeedIndex(SpeedToggle::Right),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<SpeedIndex>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Motion>>(),
		)
	}

	#[test_case(CharacterMotion::Direction { direction: Dir3::Z, speed: Speed(SLOW) }; "direction")]
	#[test_case(CharacterMotion::ToTarget { target: Vec3::Z, speed: Speed(SLOW) }; "target")]
	fn act_only_once(motion: CharacterMotion) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(motion),
				Config {
					speed: MovementSpeed::Variable([SLOW, FAST]),
					..default()
				},
				SpeedIndex(SpeedToggle::Right),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Motion>>(),
		)
	}

	#[test_case(CharacterMotion::Direction { direction: Dir3::Z, speed: Speed(SLOW) }; "direction")]
	#[test_case(CharacterMotion::ToTarget { target: Vec3::Z, speed: Speed(SLOW) }; "target")]
	fn act_again_of_speed_changed(motion: CharacterMotion) {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Motion(motion),
				Config {
					speed: MovementSpeed::Variable([SLOW, FAST]),
					..default()
				},
				SpeedIndex(SpeedToggle::Right),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<SpeedIndex>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world().entity(entity).get::<IsChanged<_Motion>>(),
		)
	}
}
