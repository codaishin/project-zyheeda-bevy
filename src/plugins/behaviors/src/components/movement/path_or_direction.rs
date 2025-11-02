use super::Movement;
use crate::traits::MovementUpdate;
use bevy::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::{handles_movement::MovementTarget, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PathOrDirection<TMotion> {
	pub(crate) mode: Mode,
	pub(crate) _m: PhantomData<TMotion>,
}

impl<TMotion> From<Option<MovementTarget>> for PathOrDirection<TMotion>
where
	TMotion: ThreadSafe,
{
	fn from(target: Option<MovementTarget>) -> Self {
		match target {
			Some(MovementTarget::Point(point)) => Self {
				mode: Mode::Path(VecDeque::from([point])),
				_m: PhantomData,
			},
			Some(MovementTarget::Dir(direction)) => Self {
				mode: Mode::Direction(Some(direction)),
				_m: PhantomData,
			},
			None => Self {
				mode: Mode::Direction(None),
				_m: PhantomData,
			},
		}
	}
}

impl<TMotion> MovementUpdate for Movement<PathOrDirection<TMotion>>
where
	TMotion: ThreadSafe,
{
	type TComponents<'a> = &'a mut PathOrDirection<TMotion>;
	type TConstraint = Without<Movement<TMotion>>;

	fn update(
		&self,
		agent: &mut ZyheedaEntityCommands,
		mut path_or_direction: Mut<PathOrDirection<TMotion>>,
		_: Speed,
	) -> Done {
		let Some(wp) = next_waypoint(&mut path_or_direction) else {
			agent.try_remove::<PathOrDirection<TMotion>>();
			agent.try_insert(Movement::<TMotion>::stop());
			return Done::from(true);
		};

		agent.try_insert(Movement::<TMotion>::to(wp));

		Done::from(false)
	}
}

fn next_waypoint<TMotion>(
	path_or_direction: &mut PathOrDirection<TMotion>,
) -> Option<MovementTarget> {
	match &mut path_or_direction.mode {
		Mode::Direction(target) => target.take().map(MovementTarget::Dir),
		Mode::Path(path) => path.pop_front().map(MovementTarget::Point),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum Mode {
	Direction(Option<Dir3>),
	Path(VecDeque<Vec3>),
}

#[cfg(test)]
mod test_with_path {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
		traits::accessors::get::TryApplyOn,
		zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	fn system(
		func: impl Fn(&mut ZyheedaEntityCommands, Mut<PathOrDirection<_MoveMethod>>) -> Done,
	) -> impl Fn(ZyheedaCommands, Query<(Entity, &mut PathOrDirection<_MoveMethod>)>) -> Done {
		move |mut commands, mut query| {
			let Ok((entity, path)) = query.single_mut() else {
				return Done::from(false);
			};

			commands
				.try_apply_on(&entity, |mut e| func(&mut e, path))
				.unwrap_or_default()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	static SPEED: LazyLock<Speed> = LazyLock::new(|| Speed(UnitsPerSecond::from(42.)));

	mod path {
		use super::*;

		#[test]
		fn insert_movement_from_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([wp, Vec3::default()])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&Movement::<_MoveMethod>::to(wp)),
				app.world().entity(entity).get::<Movement<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn dequeue_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let other = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([Vec3::new(-1., -2., -3.), other])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([other])),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_path_can_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			app.world_mut().spawn(PathOrDirection::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([wp])),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_path_can_not_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(PathOrDirection::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([])),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn insert_stop_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&Movement::stop()),
				app.world().entity(entity).get::<Movement<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn remove_path_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				None,
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
			Ok(())
		}
	}

	mod wasd {
		use super::*;

		#[test]
		fn insert_movement_from_wasd() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Dir3::NEG_Z;
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Direction(Some(wp)),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&Movement::<_MoveMethod>::to(wp)),
				app.world().entity(entity).get::<Movement<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn take_wasd_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Direction(Some(Dir3::NEG_Z)),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Direction(None),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_wasd_has_some_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Dir3::NEG_Z;
			app.world_mut().spawn(PathOrDirection::<_MoveMethod> {
				mode: Mode::Direction(Some(wp)),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_wasd_has_no_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(PathOrDirection::<_MoveMethod> {
				mode: Mode::Direction(None),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn remove_wasd_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrDirection::<_MoveMethod> {
					mode: Mode::Direction(None),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrDirection<_MoveMethod>>::stop();
					movement.update(entity, components, *SPEED)
				}))?;

			assert_eq!(
				None,
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
			Ok(())
		}
	}
}
