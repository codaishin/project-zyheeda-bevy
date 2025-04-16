use super::Movement;
use crate::{
	events::{MoveClickEvent, MoveWasdEvent},
	traits::{IsDone, MovementUpdate},
};
use bevy::{ecs::query::QueryItem, prelude::*};
use common::{
	tools::UnitsPerSecond,
	traits::{thread_safe::ThreadSafe, try_remove_from::TryRemoveFrom},
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AlongPath<TMoveMethod> {
	pub(crate) path: VecDeque<Vec3>,
	pub(crate) _m: PhantomData<TMoveMethod>,
}

impl<TMoveMethod> AlongPath<TMoveMethod> {
	pub(crate) fn new_path() -> Self {
		Self::default()
	}

	pub(crate) fn new_wasd() -> Self {
		Self::default()
	}
}

impl<TMoveMethod> Default for AlongPath<TMoveMethod> {
	fn default() -> Self {
		Self {
			path: Default::default(),
			_m: Default::default(),
		}
	}
}

impl<TMoveMethod> From<&MoveClickEvent> for Movement<AlongPath<TMoveMethod>> {
	fn from(MoveClickEvent(target): &MoveClickEvent) -> Self {
		Self {
			target: *target,
			cstr: AlongPath::new_path,
		}
	}
}

impl<TMoveMethod> From<&MoveWasdEvent> for Movement<AlongPath<TMoveMethod>> {
	fn from(MoveWasdEvent(target): &MoveWasdEvent) -> Self {
		Self {
			target: *target,
			cstr: AlongPath::new_wasd,
		}
	}
}

impl<TMoveMethod> AlongPath<TMoveMethod>
where
	TMoveMethod: ThreadSafe,
{
	pub(crate) fn with_path(path: &[Vec3]) -> Self {
		Self {
			path: VecDeque::from_iter(path.iter().copied()),
			_m: PhantomData,
		}
	}

	pub(crate) fn cleanup(
		mut commands: Commands,
		mut removed_paths: RemovedComponents<Movement<Self>>,
	) {
		for entity in removed_paths.read() {
			commands.try_remove_from::<Movement<TMoveMethod>>(entity);
		}
	}
}

impl<TMoveMethod> MovementUpdate for Movement<AlongPath<TMoveMethod>>
where
	TMoveMethod: ThreadSafe + Default,
{
	type TComponents<'a> = &'a mut AlongPath<TMoveMethod>;
	type TConstraint = Without<Movement<TMoveMethod>>;

	fn update(
		&self,
		agent: &mut EntityCommands,
		mut path: QueryItem<Self::TComponents<'_>>,
		_: UnitsPerSecond,
	) -> IsDone {
		let Some(target) = path.path.pop_front() else {
			agent.remove::<AlongPath<TMoveMethod>>();
			return IsDone(true);
		};

		agent.try_insert(Movement {
			target,
			cstr: TMoveMethod::default,
		});

		IsDone(false)
	}
}

#[cfg(test)]
mod test_with_path {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod(Vec3);

	impl From<Vec3> for _MoveMethod {
		fn from(value: Vec3) -> Self {
			_MoveMethod(value)
		}
	}

	fn system(
		func: impl Fn(&mut EntityCommands, QueryItem<&mut AlongPath<_MoveMethod>>) -> IsDone,
	) -> impl Fn(Commands, Query<(Entity, &mut AlongPath<_MoveMethod>)>) -> IsDone {
		move |mut commands, mut query| {
			let Ok((entity, path)) = query.get_single_mut() else {
				return IsDone(false);
			};

			let Some(mut entity) = commands.get_entity(entity) else {
				return IsDone(false);
			};

			func(&mut entity, path)
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_from_path() -> Result<(), RunSystemError> {
		let mut app = setup();
		let wp = Vec3::new(1., 2., 3.);
		let entity = app
			.world_mut()
			.spawn(AlongPath::<_MoveMethod> {
				path: VecDeque::from([wp, Vec3::default()]),
				_m: PhantomData,
			})
			.id();

		app.world_mut()
			.run_system_once(system(move |entity, components| {
				let movement = Movement {
					target: default(),
					cstr: AlongPath::new_path,
				};
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(
			Some(&Movement {
				target: wp,
				cstr: _MoveMethod::default
			}),
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
			.spawn(AlongPath::<_MoveMethod> {
				path: VecDeque::from([Vec3::new(-1., -2., -3.), other]),
				_m: PhantomData,
			})
			.id();

		app.world_mut()
			.run_system_once(system(move |entity, components| {
				let movement = Movement {
					target: default(),
					cstr: AlongPath::new_path,
				};
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(
			Some(&AlongPath::<_MoveMethod> {
				path: VecDeque::from([other]),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
		Ok(())
	}

	#[test]
	fn is_not_done_when_path_can_be_dequeued() -> Result<(), RunSystemError> {
		let mut app = setup();
		let wp = Vec3::new(1., 2., 3.);
		app.world_mut().spawn(AlongPath::<_MoveMethod> {
			path: VecDeque::from([wp]),
			_m: PhantomData,
		});

		let is_done = app
			.world_mut()
			.run_system_once(system(|entity, components| {
				let movement = Movement {
					target: default(),
					cstr: AlongPath::new_path,
				};
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(IsDone(false), is_done);
		Ok(())
	}

	#[test]
	fn is_done_when_path_can_not_be_dequeued() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(AlongPath::<_MoveMethod> {
			path: VecDeque::from([]),
			_m: PhantomData,
		});

		let is_done = app
			.world_mut()
			.run_system_once(system(|entity, components| {
				let movement = Movement {
					target: default(),
					cstr: AlongPath::new_path,
				};
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(IsDone(true), is_done);
		Ok(())
	}

	#[test]
	fn remove_path_when_done() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AlongPath::<_MoveMethod> {
				path: VecDeque::from([]),
				_m: PhantomData,
			})
			.id();

		app.world_mut()
			.run_system_once(system(|entity, components| {
				let movement = Movement {
					target: default(),
					cstr: AlongPath::new_path,
				};
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(
			None,
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
		Ok(())
	}
}

#[cfg(test)]
mod test_cleanup {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Debug, PartialEq, Default)]
	struct _Movement;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, AlongPath::<_Movement>::cleanup);

		app
	}

	#[test]
	fn remove_movement_when_path_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement {
					target: Vec3::default(),
					cstr: AlongPath::<_Movement>::new_path,
				},
				Movement {
					target: Vec3::default(),
					cstr: _Movement::default,
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<Movement<AlongPath<_Movement>>>();
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<Movement<_Movement>>()
		);
	}
	#[test]
	fn do_not_remove_movement_when_path_not_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement {
					target: Vec3::default(),
					cstr: AlongPath::<_Movement>::new_path,
				},
				Movement {
					target: Vec3::default(),
					cstr: _Movement::default,
				},
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&Movement {
				target: Vec3::default(),
				cstr: _Movement::default
			}),
			app.world().entity(entity).get::<Movement<_Movement>>()
		);
	}
}
