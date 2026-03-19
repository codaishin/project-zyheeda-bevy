use crate::components::new_movement::NewMovement;
use bevy::{
	ecs::query::{QueryData, QueryFilter, QueryItem},
	prelude::*,
};
use common::{
	tools::{Done, speed::Speed},
	traits::{
		accessors::get::{DynProperty, GetProperty},
		handles_movement::MovementTarget,
		handles_physics::CharacterMotion,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};

pub(crate) trait MovementUpdate {
	type TComponents: QueryData;
	type TConstraint: QueryFilter;

	fn update(
		entity: &mut ZyheedaEntityCommands,
		components: QueryItem<Self::TComponents>,
		speed: Speed,
	) -> Done;

	fn stop(entity: &mut ZyheedaEntityCommands);
}

impl<TMotion> MovementUpdate for TMotion
where
	TMotion: From<CharacterMotion> + GetProperty<Done> + GetProperty<CharacterMotion> + Component,
{
	type TComponents = (&'static NewMovement, Option<&'static TMotion>);
	type TConstraint = ();

	fn update(
		agent: &mut ZyheedaEntityCommands,
		(movement, motion): (&NewMovement, Option<&TMotion>),
		speed: Speed,
	) -> Done {
		let new_motion = match *movement {
			NewMovement::Stopped => CharacterMotion::Stop,
			NewMovement::Target(MovementTarget::Point(target)) => {
				CharacterMotion::ToTarget { target, speed }
			}
			NewMovement::Target(MovementTarget::Dir(direction)) => {
				CharacterMotion::Direction { direction, speed }
			}
		};

		match motion {
			Some(motion) if motion.dyn_property::<CharacterMotion>() == new_motion => {
				Done(motion.dyn_property::<Done>())
			}
			_ => {
				agent.try_insert(TMotion::from(new_motion));
				Done::when(new_motion == CharacterMotion::Stop)
			}
		}
	}

	fn stop(entity: &mut ZyheedaEntityCommands) {
		entity.try_insert(NewMovement::Stopped);
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

	impl GetProperty<Done> for _Motion {
		fn get_property(&self) -> bool {
			matches!(self, _Motion::Done(..))
		}
	}

	impl GetProperty<CharacterMotion> for _Motion {
		fn get_property(&self) -> CharacterMotion {
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
		agents: Query<(Entity, &NewMovement, Option<&_Motion>, &_Speed)>,
	) {
		for (entity, movement, motion, _Speed(speed)) in &agents {
			commands.try_apply_on(&entity, |mut e| {
				let result = _Motion::update(&mut e, (movement, motion), *speed);
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
			.spawn((NewMovement::to(target), _Speed(speed)))
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
			.spawn((NewMovement::to(direction), _Speed(speed)))
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
				NewMovement::to(target),
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
				NewMovement::to(target),
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
				NewMovement::to(target),
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
				NewMovement::to(target),
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
			.spawn((NewMovement::to(target), _Speed(speed)))
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
				NewMovement::to(target),
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
				NewMovement::to(target),
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

	#[test]
	fn update_returns_done_when_inserting_stopped_movement() {
		let mut app = setup();
		let speed = Speed(UnitsPerSecond::from(11.));
		let agent = app
			.world_mut()
			.spawn((
				NewMovement::Stopped,
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
