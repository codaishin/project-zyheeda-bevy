use super::Movement;
use crate::traits::{IsDone, MovementUpdate};
use bevy::{ecs::query::QueryItem, prelude::*};
use common::{
	tools::UnitsPerSecond,
	traits::{
		handles_path_finding::ComputePath,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct AlongPath<TMoveMethod> {
	path: VecDeque<Vec3>,
	_m: PhantomData<TMoveMethod>,
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

	#[allow(clippy::type_complexity)]
	pub(crate) fn new_path<TComputer>(
		mut commands: Commands,
		mut movements: Query<(Entity, &GlobalTransform, &Movement<Self>), Changed<Movement<Self>>>,
		computers: Query<&TComputer>,
	) where
		TComputer: Component + ComputePath,
	{
		if movements.is_empty() {
			return;
		}

		let Ok(computer) = computers.get_single() else {
			return;
		};

		for (entity, transform, movement) in &mut movements {
			let start = transform.translation();
			let end = movement.target;
			let Some(path) = computer.compute_path(start, end) else {
				continue;
			};
			let path = match path.as_slice() {
				[_, without_start @ ..] => without_start,
				empty => empty,
			};

			commands.try_insert_on(entity, Self::with_path(path));
			commands.try_remove_from::<Movement<TMoveMethod>>(entity);
		}
	}
}

impl<TMoveMethod> MovementUpdate for Movement<AlongPath<TMoveMethod>>
where
	TMoveMethod: ThreadSafe,
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

		agent.try_insert(Movement::<TMoveMethod>::to(target));

		IsDone(false)
	}
}

#[cfg(test)]
mod test_new_path {
	use super::*;
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq)]
	struct _MoveMethod;

	#[derive(Component, NestedMocks)]
	struct _ComputePath {
		mock: Mock_ComputePath,
	}

	#[automock]
	impl ComputePath for _ComputePath {
		fn compute_path(&self, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
			self.mock.compute_path(start, end)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, AlongPath::<_MoveMethod>::new_path::<_ComputePath>);

		app
	}

	#[test]
	fn set_path_ignoring_first() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement::<AlongPath<_MoveMethod>>::to(Vec3::default()),
				GlobalTransform::default(),
			))
			.id();
		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().return_const(Some(vec![
				Vec3::splat(1.),
				Vec3::splat(2.),
				Vec3::splat(3.),
			]));
		}));

		app.update();

		assert_eq!(
			Some(&AlongPath::<_MoveMethod> {
				path: VecDeque::from([Vec3::splat(2.), Vec3::splat(3.)]),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
	}

	#[test]
	fn no_panic_if_path_len_zero() {
		let mut app = setup();
		app.world_mut().spawn((
			Movement::<AlongPath<_MoveMethod>>::to(Vec3::default()),
			GlobalTransform::default(),
		));
		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().return_const(Some(vec![]));
		}));

		app.update();
	}

	#[test]
	fn remove_present_movement() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement::<AlongPath<_MoveMethod>>::to(Vec3::default()),
				GlobalTransform::default(),
				Movement::<_MoveMethod>::to(Vec3::default()),
			))
			.id();
		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().return_const(Some(vec![
				Vec3::splat(1.),
				Vec3::splat(2.),
				Vec3::splat(3.),
			]));
		}));

		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<Movement::<_MoveMethod>>()
		);
	}

	#[test]
	fn compute_path_correctly() {
		let mut app = setup();
		app.world_mut().spawn((
			Movement::<AlongPath<_MoveMethod>>::to(Vec3::new(4., 5., 6.)),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path()
				.times(1)
				.with(eq(Vec3::new(1., 2., 3.)), eq(Vec3::new(4., 5., 6.)))
				.return_const(None);
		}));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_changed() {
		let mut app = setup();
		app.world_mut().spawn((
			Movement::<AlongPath<_MoveMethod>>::to(Vec3::new(4., 5., 6.)),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().times(1).return_const(None);
		}));

		app.update();
		app.update();
	}

	#[test]
	fn compute_again_if_movement_mutably_dereferenced() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Movement::<AlongPath<_MoveMethod>>::to(Vec3::new(4., 5., 6.)),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().times(2).return_const(None);
		}));

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Movement<AlongPath<_MoveMethod>>>()
			.as_deref_mut();
		app.update();
	}
}

#[cfg(test)]
mod test_movement {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

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
				let movement = Movement::<AlongPath<_MoveMethod>>::default();
				movement.update(entity, components, UnitsPerSecond::new(42.))
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
			.spawn(AlongPath::<_MoveMethod> {
				path: VecDeque::from([Vec3::new(-1., -2., -3.), other]),
				_m: PhantomData,
			})
			.id();

		app.world_mut()
			.run_system_once(system(move |entity, components| {
				let movement = Movement::<AlongPath<_MoveMethod>>::default();
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
				let movement = Movement::<AlongPath<_MoveMethod>>::default();
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
				let movement = Movement::<AlongPath<_MoveMethod>>::default();
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
				let movement = Movement::<AlongPath<_MoveMethod>>::default();
				movement.update(entity, components, UnitsPerSecond::new(42.))
			}))?;

		assert_eq!(
			None,
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
		Ok(())
	}
}
