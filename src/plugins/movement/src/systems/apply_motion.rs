use crate::components::{
	config::{Config, SpeedIndex},
	movement::Movement,
};
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::{
		accessors::get::{TryApplyOn, View},
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Movement {
	pub(crate) fn apply<TMotion>(
		mut commands: ZyheedaCommands,
		movements: Query<(Entity, &mut Self, &Config, &SpeedIndex, Option<&TMotion>)>,
	) where
		TMotion: Component + From<CharacterMotion> + View<CharacterMotion>,
	{
		for (entity, mut movement, config, speed_index, current_motion) in movements {
			let Some(motion) = movement.get_motion(config, speed_index, current_motion) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(TMotion::from(motion));
			});
		}
	}

	fn get_motion<TMotion>(
		&mut self,
		config: &Config,
		speed_index: &SpeedIndex,
		current_motion: Option<&TMotion>,
	) -> Option<CharacterMotion>
	where
		TMotion: View<CharacterMotion>,
	{
		let current_motion = current_motion.map(|m| m.view());

		match self {
			Movement::None => match current_motion {
				Some(CharacterMotion::Done) => None,
				_ => Some(CharacterMotion::Done),
			},
			Movement::Direction(direction) => {
				let motion = CharacterMotion::Direction {
					speed: Speed(config[*speed_index]),
					direction: *direction,
				};

				match current_motion {
					Some(current_motion) if current_motion == motion => None,
					_ => Some(motion),
				}
			}
			Movement::Target(target) => {
				let motion = CharacterMotion::ToTarget {
					speed: Speed(config[*speed_index]),
					target: *target,
				};

				match current_motion {
					Some(current_motion) if current_motion == motion => None,
					_ => Some(motion),
				}
			}
			Movement::Path(path) if path.is_new() || is_none_or_done(current_motion) => {
				Some(CharacterMotion::ToTarget {
					speed: Speed(config[*speed_index]),
					target: path.pop_front()?,
				})
			}
			_ => None,
		}
	}
}

fn is_none_or_done(motion: Option<CharacterMotion>) -> bool {
	matches!(motion, Some(CharacterMotion::Done) | None)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		config::{Config, SpeedIndex},
		movement::MovementPath,
	};
	use common::{
		tools::{UnitsPerSecond, speed::Speed},
		traits::handles_movement::{MovementSpeed, SpeedToggle},
	};
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Component, Debug, PartialEq)]
	struct _Motion(CharacterMotion);

	impl From<CharacterMotion> for _Motion {
		fn from(motion: CharacterMotion) -> Self {
			Self(motion)
		}
	}

	impl View<CharacterMotion> for _Motion {
		fn view(&self) -> CharacterMotion {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(Movement::apply::<_Motion>, IsChanged::<_Motion>::detect).chain(),
		);

		app
	}

	const SLOW: UnitsPerSecond = UnitsPerSecond::from_u8(2);
	const FAST: UnitsPerSecond = UnitsPerSecond::from_u8(11);

	mod without_motion {

		use super::*;
		use test_case::test_case;

		#[test]
		fn apply_stop_when_none() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::None,
					SpeedIndex(SpeedToggle::Right),
					Config::default(),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::Done)),
				app.world().entity(entity).get::<_Motion>(),
			);
		}

		#[test_case(SpeedToggle::Left, SLOW; "slow")]
		#[test_case(SpeedToggle::Right, FAST; "fast")]
		fn apply_direction(toggle: SpeedToggle, expected_speed: UnitsPerSecond) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Direction(Dir3::Z),
					SpeedIndex(toggle),
					Config {
						speed: MovementSpeed::Variable([SLOW, FAST]),
						..default()
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::Direction {
					speed: Speed(expected_speed),
					direction: Dir3::Z
				})),
				app.world().entity(entity).get::<_Motion>(),
			);
		}

		#[test_case(SpeedToggle::Left, SLOW; "slow")]
		#[test_case(SpeedToggle::Right, FAST; "fast")]
		fn apply_target(toggle: SpeedToggle, expected_speed: UnitsPerSecond) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Target(Vec3::new(1., 2., 3.)),
					SpeedIndex(toggle),
					Config {
						speed: MovementSpeed::Variable([SLOW, FAST]),
						..default()
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::ToTarget {
					speed: Speed(expected_speed),
					target: Vec3::new(1., 2., 3.),
				})),
				app.world().entity(entity).get::<_Motion>(),
			);
		}

		#[test_case(SpeedToggle::Left, SLOW; "slow")]
		#[test_case(SpeedToggle::Right, FAST; "fast")]
		fn apply_path(toggle: SpeedToggle, expected_speed: UnitsPerSecond) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Path(MovementPath::from([
						Vec3::new(1., 2., 3.),
						Vec3::new(3., 4., 5.),
					])),
					SpeedIndex(toggle),
					Config {
						speed: MovementSpeed::Variable([SLOW, FAST]),
						..default()
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::ToTarget {
					speed: Speed(expected_speed),
					target: Vec3::new(1., 2., 3.),
				})),
				app.world().entity(entity).get::<_Motion>(),
			);
		}

		#[test]
		fn apply_nothing_when_path_empty() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Path(MovementPath::from([])),
					SpeedIndex(SpeedToggle::Right),
					Config::default(),
				))
				.id();

			app.update();

			assert_eq!(None, app.world().entity(entity).get::<_Motion>());
		}

		#[test]
		fn dequeue_path() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Path(MovementPath::from([
						Vec3::new(1., 2., 3.),
						Vec3::new(3., 4., 5.),
					])),
					SpeedIndex(SpeedToggle::Left),
					Config::default(),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::Path(
					MovementPath::from([Vec3::new(3., 4., 5.)]).not_new()
				)),
				app.world().entity(entity).get::<Movement>(),
			);
		}
	}

	mod with_motion {
		use super::*;
		use test_case::test_case;

		#[test]
		fn do_not_stop_when_motion_stopped() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::None,
					SpeedIndex(SpeedToggle::Left),
					Config {
						speed: MovementSpeed::Fixed(SLOW),
						..default()
					},
				))
				.id();

			app.update();
			app.update();

			assert_eq!(
				Some(&IsChanged::FALSE),
				app.world().entity(entity).get::<IsChanged<_Motion>>(),
			);
		}

		#[test]
		fn stop_when_in_motion() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Motion::from(CharacterMotion::Direction {
						speed: Speed(SLOW),
						direction: Dir3::Y,
					}),
					Movement::None,
					SpeedIndex(SpeedToggle::Left),
					Config {
						speed: MovementSpeed::Fixed(SLOW),
						..default()
					},
				))
				.id();

			app.update();

			assert_eq!(
				Some(&_Motion::from(CharacterMotion::Done)),
				app.world().entity(entity).get::<_Motion>(),
			);
		}

		#[test_case([Dir3::Z, Dir3::Z], IsChanged::FALSE; "not when same direction")]
		#[test_case([Dir3::Z, Dir3::X], IsChanged::TRUE; "when motion changed")]
		fn reapply_direction(directions: [Dir3; 2], is_changed: IsChanged<_Motion>) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Direction(directions[0]),
					SpeedIndex(SpeedToggle::Left),
					Config {
						speed: MovementSpeed::Fixed(SLOW),
						..default()
					},
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.insert(Movement::Direction(directions[1]));
			app.update();

			assert_eq!(
				Some(&is_changed),
				app.world().entity(entity).get::<IsChanged<_Motion>>(),
			);
		}

		#[test_case([Vec3::Z, Vec3::Z], IsChanged::FALSE; "not when same target")]
		#[test_case([Vec3::Z, Vec3::X], IsChanged::TRUE; "when motion changed")]
		fn reapply_target(targets: [Vec3; 2], is_changed: IsChanged<_Motion>) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Movement::Target(targets[0]),
					SpeedIndex(SpeedToggle::Left),
					Config {
						speed: MovementSpeed::Fixed(SLOW),
						..default()
					},
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.insert(Movement::Target(targets[1]));
			app.update();

			assert_eq!(
				Some(&is_changed),
				app.world().entity(entity).get::<IsChanged<_Motion>>(),
			);
		}

		mod path_not_new {
			use super::*;

			#[test]
			fn do_not_apply_path_when_motion_not_done() {
				let mut app = setup();
				let entity = app
					.world_mut()
					.spawn((
						_Motion(CharacterMotion::Direction {
							speed: Speed(SLOW),
							direction: Dir3::NEG_Y,
						}),
						Movement::Path(MovementPath::from([Vec3::new(1., 2., 3.)]).not_new()),
						SpeedIndex(SpeedToggle::Left),
						Config {
							speed: MovementSpeed::Fixed(SLOW),
							..default()
						},
					))
					.id();

				app.update();

				assert_eq!(
					(
						Some(&_Motion(CharacterMotion::Direction {
							speed: Speed(SLOW),
							direction: Dir3::NEG_Y,
						})),
						Some(&Movement::Path(
							MovementPath::from([Vec3::new(1., 2., 3.)]).not_new()
						))
					),
					(
						app.world().entity(entity).get::<_Motion>(),
						app.world().entity(entity).get::<Movement>(),
					)
				);
			}
		}

		#[test]
		fn apply_path_when_motion_done() {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					_Motion(CharacterMotion::Done),
					Movement::Path(MovementPath::from([Vec3::new(1., 2., 3.)]).not_new()),
					SpeedIndex(SpeedToggle::Left),
					Config {
						speed: MovementSpeed::Fixed(SLOW),
						..default()
					},
				))
				.id();

			app.update();

			assert_eq!(
				(
					Some(&_Motion(CharacterMotion::ToTarget {
						speed: Speed(SLOW),
						target: Vec3::new(1., 2., 3.),
					})),
					Some(&Movement::Path(MovementPath::from([]).not_new()))
				),
				(
					app.world().entity(entity).get::<_Motion>(),
					app.world().entity(entity).get::<Movement>(),
				)
			);
		}

		mod path_new {
			use super::*;

			#[test]
			fn apply_path_when_motion_not_done_and_movement_new() {
				let mut app = setup();
				let entity = app
					.world_mut()
					.spawn((
						_Motion(CharacterMotion::Direction {
							speed: Speed(SLOW),
							direction: Dir3::NEG_Y,
						}),
						Movement::Path(MovementPath::from([Vec3::new(1., 2., 3.)])),
						SpeedIndex(SpeedToggle::Left),
						Config {
							speed: MovementSpeed::Fixed(SLOW),
							..default()
						},
					))
					.id();

				app.update();

				assert_eq!(
					(
						Some(&_Motion(CharacterMotion::ToTarget {
							speed: Speed(SLOW),
							target: Vec3::new(1., 2., 3.),
						})),
						Some(&Movement::Path(MovementPath::from([]).not_new()))
					),
					(
						app.world().entity(entity).get::<_Motion>(),
						app.world().entity(entity).get::<Movement>(),
					)
				);
			}
		}
	}
}
