use crate::{AlongPath, Movement};
use bevy::prelude::*;
use common::{
	tools::collider_radius::ColliderRadius,
	traits::{
		accessors::get::{GetField, Getter},
		handles_path_finding::ComputePath,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> MovementPath for T {}

type PathMovement<T> = Movement<AlongPath<T>>;
type Components<'a, TMoveMethod, TAgent> = (
	Entity,
	&'a GlobalTransform,
	&'a PathMovement<TMoveMethod>,
	&'a TAgent,
);

pub(crate) trait MovementPath {
	fn path<TMoveMethod, TComputer>(
		mut commands: Commands,
		mut movements: Query<Components<TMoveMethod, Self>, Changed<PathMovement<TMoveMethod>>>,
		computers: Query<&TComputer>,
	) where
		Self: Component + Getter<ColliderRadius> + Sized,
		TMoveMethod: ThreadSafe,
		TComputer: Component + ComputePath,
	{
		if movements.is_empty() {
			return;
		}

		let Ok(computer) = computers.get_single() else {
			return;
		};

		for (entity, transform, movement, agent) in &mut movements {
			let start = transform.translation();
			let end = movement.target;
			let ColliderRadius(radius) = ColliderRadius::get_field(agent);
			let Some(path) = computer.compute_path(start, end, radius) else {
				continue;
			};
			let path = match path.as_slice() {
				[first, rest @ ..] if first == &start => rest,
				path => path,
			};

			commands.try_insert_on(entity, AlongPath::<TMoveMethod>::with_path(path));
			commands.try_remove_from::<Movement<TMoveMethod>>(entity);
		}
	}
}

#[cfg(test)]
mod test_new_path {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::Units,
		traits::{clamp_zero_positive::ClampZeroPositive, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, marker::PhantomData};

	#[derive(Debug, PartialEq)]
	struct _MoveMethod;

	#[derive(Component, NestedMocks)]
	struct _AgentMovement {
		mock: Mock_AgentMovement,
	}

	impl Default for _AgentMovement {
		fn default() -> Self {
			let mut mock = Mock_AgentMovement::new();
			mock.expect_get()
				.return_const(ColliderRadius(Units::new(1.)));

			Self { mock }
		}
	}

	#[automock]
	impl Getter<ColliderRadius> for _AgentMovement {
		fn get(&self) -> ColliderRadius {
			self.mock.get()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _ComputePath {
		mock: Mock_ComputePath,
	}

	#[automock]
	impl ComputePath for _ComputePath {
		fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Vec<Vec3>> {
			self.mock.compute_path(start, end, agent_radius)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _AgentMovement::path::<_MoveMethod, _ComputePath>);

		app
	}

	#[test]
	fn set_path() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement {
					target: Vec3::default(),
					cstr: AlongPath::<_MoveMethod>::new_path,
				},
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
				path: VecDeque::from([Vec3::splat(1.), Vec3::splat(2.), Vec3::splat(3.)]),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
	}

	#[test]
	fn set_path_ignoring_first_when_matching_translation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement {
					target: Vec3::default(),
					cstr: AlongPath::<_MoveMethod>::new_path,
				},
				GlobalTransform::from_translation(Vec3::splat(1.)),
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
			_AgentMovement::default(),
			Movement {
				target: Vec3::default(),
				cstr: AlongPath::<_MoveMethod>::new_path,
			},
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
				_AgentMovement::default(),
				Movement {
					target: Vec3::default(),
					cstr: AlongPath::<_MoveMethod>::new_path,
				},
				GlobalTransform::default(),
				Movement {
					target: Vec3::default(),
					cstr: || _MoveMethod,
				},
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
			_AgentMovement::new().with_mock(|mock| {
				mock.expect_get()
					.return_const(ColliderRadius(Units::new(42.)));
			}),
			Movement {
				target: Vec3::new(4., 5., 6.),
				cstr: AlongPath::<_MoveMethod>::new_path,
			},
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path()
				.times(1)
				.with(
					eq(Vec3::new(1., 2., 3.)),
					eq(Vec3::new(4., 5., 6.)),
					eq(Units::new(42.)),
				)
				.return_const(None);
		}));

		app.update();
	}

	#[test]
	fn do_nothing_if_not_changed() {
		let mut app = setup();
		app.world_mut().spawn((
			_AgentMovement::default(),
			Movement {
				target: Vec3::new(4., 5., 6.),
				cstr: AlongPath::<_MoveMethod>::new_path,
			},
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
				_AgentMovement::default(),
				Movement {
					target: Vec3::new(4., 5., 6.),
					cstr: AlongPath::<_MoveMethod>::new_path,
				},
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
