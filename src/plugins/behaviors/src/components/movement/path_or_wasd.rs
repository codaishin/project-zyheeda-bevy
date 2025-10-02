use super::Movement;
use crate::{
	components::movement::MotionSpec,
	systems::movement::parse_directional_movement_key::UsesDirection,
	traits::MovementUpdate,
};
use bevy::prelude::*;
use common::{
	tools::{Done, speed::Speed},
	traits::{handles_movement_behavior::PathMotionSpec, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaEntityCommands,
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PathOrWasd<TMotion> {
	pub(crate) mode: Mode,
	pub(crate) _m: PhantomData<TMotion>,
}

impl<TMotion> From<PathMotionSpec> for PathOrWasd<TMotion>
where
	TMotion: ThreadSafe,
{
	fn from(spec: PathMotionSpec) -> Self {
		match spec {
			PathMotionSpec(MotionSpec::ToTarget { target, .. }) => Self {
				mode: Mode::Path(VecDeque::from([target])),
				_m: PhantomData,
			},
			PathMotionSpec(MotionSpec::Direction { direction, .. }) => Self {
				mode: Mode::Wasd(Some(direction)),
				_m: PhantomData,
			},
			PathMotionSpec(MotionSpec::Stop) => Self {
				mode: Mode::Wasd(None),
				_m: PhantomData,
			},
		}
	}
}

impl<TMotion> UsesDirection for Movement<PathOrWasd<TMotion>>
where
	TMotion: ThreadSafe,
{
	fn uses_direction(&self) -> bool {
		matches!(self.spec, PathMotionSpec(MotionSpec::Direction { .. }))
	}
}

impl<TMotion> MovementUpdate for Movement<PathOrWasd<TMotion>>
where
	TMotion: ThreadSafe,
{
	type TComponents<'a> = &'a mut PathOrWasd<TMotion>;
	type TConstraint = Without<Movement<TMotion>>;

	fn update(
		&self,
		agent: &mut ZyheedaEntityCommands,
		mut path_or_wasd: Mut<PathOrWasd<TMotion>>,
	) -> Done {
		let Some(wp) = next_waypoint(&mut path_or_wasd, self.spec) else {
			agent.try_remove::<PathOrWasd<TMotion>>();
			agent.try_insert(Movement::<TMotion>::from(PathMotionSpec(MotionSpec::Stop)));
			return Done::from(true);
		};

		agent.try_insert(Movement::<TMotion>::from(PathMotionSpec(wp)));

		Done::from(false)
	}
}

fn next_waypoint<TMotion>(
	path_or_wasd: &mut PathOrWasd<TMotion>,
	spec: PathMotionSpec,
) -> Option<MotionSpec>
where
	TMotion: ThreadSafe,
{
	let speed = match spec {
		PathMotionSpec(MotionSpec::ToTarget { speed, .. }) => speed,
		PathMotionSpec(MotionSpec::Direction { speed, .. }) => speed,
		PathMotionSpec(MotionSpec::Stop) => Speed::ZERO,
	};

	match &mut path_or_wasd.mode {
		Mode::Wasd(direction) => direction
			.take()
			.map(|direction| MotionSpec::Direction { speed, direction }),
		Mode::Path(path) => path
			.pop_front()
			.map(|target| MotionSpec::ToTarget { speed, target }),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum Mode {
	Wasd(Option<Dir3>),
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
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	fn system(
		func: impl Fn(&mut ZyheedaEntityCommands, Mut<PathOrWasd<_MoveMethod>>) -> Done,
	) -> impl Fn(ZyheedaCommands, Query<(Entity, &mut PathOrWasd<_MoveMethod>)>) -> Done {
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

	mod path {
		use super::*;

		#[test]
		fn insert_movement_from_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = PathMotionSpec(MotionSpec::ToTarget {
				speed: Speed(UnitsPerSecond::from(42.)),
				target: Vec3::new(1., 2., 3.),
			});
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([Vec3::new(1., 2., 3.), Vec3::default()])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				Some(&Movement::<_MoveMethod>::from(wp)),
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
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([Vec3::new(-1., -2., -3.), other])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				Some(&PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([other])),
					_m: PhantomData,
				}),
				app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_path_can_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			app.world_mut().spawn(PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([wp])),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_path_can_not_be_dequeued() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([])),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn insert_stop_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
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
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				None,
				app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
			);
			Ok(())
		}
	}

	mod wasd {
		use super::*;

		#[test]
		fn insert_movement_from_wasd() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = PathMotionSpec(MotionSpec::Direction {
				speed: Speed(UnitsPerSecond::from(42.)),
				direction: Dir3::NEG_Z,
			});
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(Some(Dir3::NEG_Z)),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				Some(&Movement::<_MoveMethod>::from(wp)),
				app.world().entity(entity).get::<Movement<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn take_wasd_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(Some(Dir3::NEG_Z)),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				Some(&PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(None),
					_m: PhantomData,
				}),
				app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
			);
			Ok(())
		}

		#[test]
		fn is_not_done_when_wasd_has_some_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Dir3::NEG_Z;
			app.world_mut().spawn(PathOrWasd::<_MoveMethod> {
				mode: Mode::Wasd(Some(wp)),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(Done::from(false), is_done);
			Ok(())
		}

		#[test]
		fn is_done_when_wasd_has_no_target() -> Result<(), RunSystemError> {
			let mut app = setup();
			app.world_mut().spawn(PathOrWasd::<_MoveMethod> {
				mode: Mode::Wasd(None),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(Done::from(true), is_done);
			Ok(())
		}

		#[test]
		fn remove_wasd_when_done() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(None),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::stop();
					movement.update(entity, components)
				}))?;

			assert_eq!(
				None,
				app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
			);
			Ok(())
		}
	}
}
