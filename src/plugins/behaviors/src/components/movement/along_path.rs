use bevy::prelude::*;
use common::traits::{handles_path_finding::ComputePath, thread_safe::ThreadSafe};
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
#[require(GlobalTransform)]
pub(crate) struct AlongPath<TMoveMethod> {
	end: Vec3,
	path: VecDeque<Vec3>,
	_m: PhantomData<TMoveMethod>,
}

impl<TMoveMethod> AlongPath<TMoveMethod>
where
	TMoveMethod: ThreadSafe,
{
	pub(crate) fn to(end: Vec3) -> Self {
		Self {
			end,
			path: VecDeque::from([]),
			_m: PhantomData,
		}
	}

	pub(crate) fn set_path<TComputer>(
		computers: Query<&TComputer>,
		mut paths: Query<(&mut Self, &GlobalTransform), Changed<Self>>,
	) where
		TComputer: Component + ComputePath,
	{
		let Ok(computer) = computers.get_single() else {
			return;
		};

		for (mut path, transform) in &mut paths {
			let start = transform.translation();
			let end = path.end;
			let Some(computed_path) = computer.compute_path(start, end) else {
				continue;
			};
			path.path = VecDeque::from(computed_path);
		}
	}
}

#[cfg(test)]
mod tests {
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
		app.add_systems(Update, AlongPath::<_MoveMethod>::set_path::<_ComputePath>);

		app
	}

	#[test]
	fn fill_path() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AlongPath::<_MoveMethod>::to(Vec3::default()))
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
				end: Vec3::default(),
				path: VecDeque::from([Vec3::splat(1.), Vec3::splat(2.), Vec3::splat(3.)]),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<AlongPath<_MoveMethod>>()
		);
	}

	#[test]
	fn compute_path_correctly() {
		let mut app = setup();
		app.world_mut().spawn((
			AlongPath::<_MoveMethod>::to(Vec3::new(4., 5., 6.)),
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
			AlongPath::<_MoveMethod>::to(Vec3::new(4., 5., 6.)),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().times(1).return_const(None);
		}));

		app.update();
		app.update();
	}

	#[test]
	fn compute_again_if_mutably_dereferenced() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AlongPath::<_MoveMethod>::to(Vec3::new(4., 5., 6.)),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();

		app.world_mut().spawn(_ComputePath::new().with_mock(|mock| {
			mock.expect_compute_path().times(2).return_const(None);
		}));

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<AlongPath<_MoveMethod>>()
			.as_deref_mut();
		app.update();
	}
}
