use super::Movement;
use crate::{
	PointerInput,
	WasdInput,
	traits::{IsDone, MovementUpdate},
};
use bevy::prelude::*;
use common::{
	tools::speed::Speed,
	traits::{accessors::get::TryApplyOn, thread_safe::ThreadSafe},
	zyheeda_commands::ZyheedaCommands,
};
use std::{collections::VecDeque, marker::PhantomData, time::Duration};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct PathOrWasd<TMethod> {
	pub(crate) mode: Mode,
	pub(crate) _m: PhantomData<TMethod>,
}

impl<TMethod> PathOrWasd<TMethod> {
	pub(crate) fn new_path() -> Self {
		Self {
			_m: PhantomData,
			mode: Mode::Path(VecDeque::default()),
		}
	}

	pub(crate) fn new_wasd() -> Self {
		Self {
			_m: PhantomData,
			mode: Mode::Wasd(None),
		}
	}
}

impl<TMethod> Default for PathOrWasd<TMethod>
where
	TMethod: ThreadSafe,
{
	fn default() -> Self {
		Self {
			mode: Mode::Path(VecDeque::default()),
			_m: Default::default(),
		}
	}
}

impl<TMethod> From<PointerInput<TMethod>> for Movement<PathOrWasd<TMethod>>
where
	TMethod: ThreadSafe,
{
	fn from(PointerInput { target, .. }: PointerInput<TMethod>) -> Self {
		Self {
			target,
			method_cstr: PathOrWasd::new_path,
		}
	}
}

impl<TMethod> From<WasdInput<TMethod>> for Movement<PathOrWasd<TMethod>>
where
	TMethod: ThreadSafe,
{
	fn from(WasdInput { target, .. }: WasdInput<TMethod>) -> Self {
		Self {
			target,
			method_cstr: PathOrWasd::new_wasd,
		}
	}
}

impl<TMethod> PathOrWasd<TMethod>
where
	TMethod: ThreadSafe + Default,
{
	pub(crate) fn cleanup(
		mut commands: ZyheedaCommands,
		mut removed_paths: RemovedComponents<Movement<Self>>,
	) {
		for entity in removed_paths.read() {
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Movement<TMethod>>();
			});
		}
	}
}

impl<TMethod> MovementUpdate for Movement<PathOrWasd<TMethod>>
where
	TMethod: ThreadSafe + Default,
{
	type TComponents<'a> = &'a mut PathOrWasd<TMethod>;
	type TConstraint = Without<Movement<TMethod>>;

	fn update(
		&self,
		agent: &mut EntityCommands,
		mut path_or_wasd: Mut<PathOrWasd<TMethod>>,
		_: Speed,
		_: Duration,
	) -> IsDone {
		let Some(wp) = next_waypoint(&mut path_or_wasd) else {
			agent.remove::<PathOrWasd<TMethod>>();
			return IsDone(true);
		};

		agent.try_insert(Movement::<TMethod>::to(wp));

		IsDone(false)
	}
}

fn next_waypoint<TMethod>(path_or_wasd: &mut PathOrWasd<TMethod>) -> Option<Vec3>
where
	TMethod: ThreadSafe,
{
	match &mut path_or_wasd.mode {
		Mode::Wasd(target) => target.take(),
		Mode::Path(path) => path.pop_front(),
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum Mode {
	Wasd(Option<Vec3>),
	Path(VecDeque<Vec3>),
}

#[cfg(test)]
mod test_with_path {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::UnitsPerSecond;
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod(Vec3);

	impl From<Vec3> for _MoveMethod {
		fn from(value: Vec3) -> Self {
			_MoveMethod(value)
		}
	}

	fn system(
		func: impl Fn(&mut EntityCommands, Mut<PathOrWasd<_MoveMethod>>) -> IsDone,
	) -> impl Fn(Commands, Query<(Entity, &mut PathOrWasd<_MoveMethod>)>) -> IsDone {
		move |mut commands, mut query| {
			let Ok((entity, path)) = query.single_mut() else {
				return IsDone(false);
			};

			let Ok(mut entity) = commands.get_entity(entity) else {
				return IsDone(false);
			};

			func(&mut entity, path)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	static SPEED: LazyLock<Speed> = LazyLock::new(|| Speed(UnitsPerSecond::from(42.)));
	static DELTA: LazyLock<Duration> = LazyLock::new(Duration::default);

	mod path {
		use super::*;

		#[test]
		fn insert_movement_from_path() -> Result<(), RunSystemError> {
			let mut app = setup();
			let wp = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([wp, Vec3::default()])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
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
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([Vec3::new(-1., -2., -3.), other])),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
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
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
				}))?;

			assert_eq!(IsDone(false), is_done);
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
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
				}))?;

			assert_eq!(IsDone(true), is_done);
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
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
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
			let wp = Vec3::new(1., 2., 3.);
			let entity = app
				.world_mut()
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(Some(wp)),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
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
				.spawn(PathOrWasd::<_MoveMethod> {
					mode: Mode::Wasd(Some(Vec3::default())),
					_m: PhantomData,
				})
				.id();

			app.world_mut()
				.run_system_once(system(move |entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
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
			let wp = Vec3::new(1., 2., 3.);
			app.world_mut().spawn(PathOrWasd::<_MoveMethod> {
				mode: Mode::Wasd(Some(wp)),
				_m: PhantomData,
			});

			let is_done = app
				.world_mut()
				.run_system_once(system(|entity, components| {
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
				}))?;

			assert_eq!(IsDone(false), is_done);
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
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
				}))?;

			assert_eq!(IsDone(true), is_done);
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
					let movement = Movement::<PathOrWasd<_MoveMethod>>::default();
					movement.update(entity, components, *SPEED, *DELTA)
				}))?;

			assert_eq!(
				None,
				app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
			);
			Ok(())
		}
	}
}

#[cfg(test)]
mod test_cleanup {
	use super::*;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, PathOrWasd::<_MoveMethod>::cleanup);

		app
	}

	#[test]
	fn remove_movement_when_path_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement::<PathOrWasd<_MoveMethod>>::default(),
				Movement::<_MoveMethod>::default(),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Movement<PathOrWasd<_MoveMethod>>>();
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<Movement<_MoveMethod>>()
		);
	}
	#[test]
	fn do_not_remove_movement_when_path_not_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement::<PathOrWasd<_MoveMethod>>::default(),
				Movement::<_MoveMethod>::default(),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&Movement::default()),
			app.world().entity(entity).get::<Movement<_MoveMethod>>()
		);
	}

	#[test]
	fn defaults_to_path() {
		let movement = PathOrWasd::<_MoveMethod>::default();

		assert_eq!(
			PathOrWasd {
				mode: Mode::Path(VecDeque::default()),
				_m: PhantomData::<_MoveMethod>,
			},
			movement,
		);
	}
}
