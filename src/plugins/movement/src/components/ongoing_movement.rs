use crate::traits::movement_update::MovementUpdate;
use bevy::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::{
		accessors::get::{View, ViewOf},
		handles_movement::MovementTarget,
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "ongoing movement")]
pub(crate) enum OngoingMovement {
	/// No movement, [`IsMoving`] marker will be removed
	#[default]
	Stopped,
	/// Transition to [`Self::Stopped`], [`IsMoving`] will be left unmodified
	Stop,
	/// Ongoing movement to target, [`IsMoving`] marker will be added
	Target(MovementTarget),
}

impl OngoingMovement {
	pub(crate) fn target(target: impl Into<MovementTarget>) -> Self {
		Self::Target(target.into())
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct IsMoving;

impl<TMotion> MovementUpdate for (OngoingMovement, TMotion)
where
	TMotion: From<CharacterMotion> + View<Done> + View<CharacterMotion> + Component,
{
	type TComponents = (&'static OngoingMovement, Option<&'static TMotion>);

	fn update(
		agent: &mut ZyheedaEntityCommands,
		(movement, motion): (&OngoingMovement, Option<&TMotion>),
		speed: Speed,
	) -> Done {
		let new_motion = match *movement {
			OngoingMovement::Stopped | OngoingMovement::Stop => CharacterMotion::Stop,
			OngoingMovement::Target(MovementTarget::Point(target)) => {
				CharacterMotion::ToTarget { target, speed }
			}
			OngoingMovement::Target(MovementTarget::Dir(direction)) => {
				CharacterMotion::Direction { direction, speed }
			}
		};

		match motion {
			Some(motion) if motion.view_of::<CharacterMotion>() == new_motion => {
				Done(motion.view_of::<Done>())
			}
			_ => {
				agent.try_insert(TMotion::from(new_motion));
				Done::when(new_motion == CharacterMotion::Stop)
			}
		}
	}

	fn stop(entity: &mut ZyheedaEntityCommands) {
		entity.try_insert(OngoingMovement::Stopped);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::UnitsPerSecond,
		traits::accessors::get::TryApplyOn,
		zyheeda_commands::ZyheedaCommands,
	};
	use test_case::test_case;
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	enum _Motion {
		NotDone(CharacterMotion),
		Done(CharacterMotion),
	}

	impl From<CharacterMotion> for _Motion {
		fn from(linear: CharacterMotion) -> Self {
			Self::NotDone(linear)
		}
	}

	impl View<Done> for _Motion {
		fn view(&self) -> bool {
			matches!(self, _Motion::Done(..))
		}
	}

	impl View<CharacterMotion> for _Motion {
		fn view(&self) -> CharacterMotion {
			match self {
				_Motion::NotDone(motion) => *motion,
				_Motion::Done(motion) => *motion,
			}
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Result(Done);

	#[derive(Component)]
	struct _Speed(Speed);

	#[allow(clippy::type_complexity)]
	fn call_update(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &OngoingMovement, Option<&_Motion>, &_Speed)>,
	) {
		for (entity, movement, motion, _Speed(speed)) in &agents {
			commands.try_apply_on(&entity, |mut e| {
				let result =
					<(OngoingMovement, _Motion)>::update(&mut e, (movement, motion), *speed);
				e.try_insert(_Result(result));
			});
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, (call_update, IsChanged::<_Motion>::detect).chain());

		app
	}

	#[test]
	fn update_applies_target_motion() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((OngoingMovement::target(target), _Speed(speed)))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(CharacterMotion::ToTarget { speed, target })),
			app.world().entity(agent).get::<_Motion>()
		);
	}
	#[test]
	fn update_applies_directional_motion() {
		let mut app = setup();
		let direction = Dir3::NEG_X;
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((OngoingMovement::target(direction), _Speed(speed)))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(CharacterMotion::Direction {
				speed,
				direction
			})),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn update_applies_motion_when_different_motion_present() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::NotDone(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(42.)),
					target: Vec3::new(1., 2., 3.),
				}),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Motion::from(CharacterMotion::ToTarget { speed, target })),
			app.world().entity(agent).get::<_Motion>()
		);
	}

	#[test]
	fn update_applies_no_motion_when_same_motion_present() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::Done(CharacterMotion::ToTarget { speed, target }),
				_Speed(speed),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(agent).get::<IsChanged<_Motion>>(),
		);
	}

	#[test]
	fn update_returns_not_done_when_target_motion_present() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::from(CharacterMotion::ToTarget {
					speed: Speed::default(),
					target: Vec3::default(),
				}),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done::from(false))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_returns_not_done_when_directional_motion_present() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::from(CharacterMotion::Direction {
					speed: Speed::default(),
					direction: Dir3::NEG_X,
				}),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done::from(false))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_returns_not_done_when_no_motion_present() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((OngoingMovement::target(target), _Speed(speed)))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done(false))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_returns_done_when_motion_done() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::Done(CharacterMotion::ToTarget { speed, target }),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done(true))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test]
	fn update_returns_not_done_when_inserting_movement_with_different_target() {
		let mut app = setup();
		let target = Vec3::new(10., 0., 7.);
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				OngoingMovement::target(target),
				_Motion::Done(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(42.)),
					target: Vec3::new(11., 1., 8.),
				}),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done(false))),
			app.world().entity(agent).get::<_Result>()
		);
	}

	#[test_case(OngoingMovement::Stop; "stop")]
	#[test_case(OngoingMovement::Stopped; "stopped")]
	fn update_returns_done_when_inserting_stopped_movement(movement: OngoingMovement) {
		let mut app = setup();
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				movement,
				_Motion::NotDone(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(42.)),
					target: Vec3::new(11., 1., 8.),
				}),
				_Speed(speed),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Result(Done(true))),
			app.world().entity(agent).get::<_Result>()
		);
	}
}
