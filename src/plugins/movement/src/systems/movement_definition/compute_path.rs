use crate::{
	Movement,
	PathOrDirection,
	components::{movement::path_or_direction::Mode, movement_definition::MovementDefinition},
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, TryApplyOn},
		handles_path_finding::ComputePath,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::VecDeque;

type MoveComponents<TMotion, TGetComputer> = (
	Entity,
	&'static MovementDefinition,
	&'static GlobalTransform,
	&'static Movement<PathOrDirection<TMotion>>,
	&'static TGetComputer,
);
type ChangedMovement<TMotion> = Changed<Movement<PathOrDirection<TMotion>>>;

impl MovementDefinition {
	pub(crate) fn compute_path<TMotion, TComputer, TGetComputer>(
		mut commands: ZyheedaCommands,
		movements: Query<MoveComponents<TMotion, TGetComputer>, ChangedMovement<TMotion>>,
		computers: Query<&TComputer>,
	) where
		TMotion: ThreadSafe,
		TComputer: Component + ComputePath,
		TGetComputer: Component + GetProperty<Entity>,
	{
		for (entity, definition, transform, movement, get_computer) in &movements {
			let Ok(computer) = computers.get(get_computer.get_property()) else {
				continue;
			};
			let path_or_direction = definition.new_movement(computer, transform, movement);
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(path_or_direction);
				e.try_remove::<Movement<TMotion>>();
			});
		}
	}

	fn new_movement<TMotion, TComputer>(
		&self,
		computer: &TComputer,
		transform: &GlobalTransform,
		movement: &Movement<PathOrDirection<TMotion>>,
	) -> PathOrDirection<TMotion>
	where
		TComputer: ComputePath,
		TMotion: ThreadSafe,
	{
		let mut new_movement = PathOrDirection::<TMotion>::from(movement.target);

		let Mode::Path(move_path) = &mut new_movement.mode else {
			return new_movement;
		};

		let ([end], []) = move_path.as_slices() else {
			return new_movement;
		};

		*move_path = self.compute_path_internal(computer, transform, *end);

		new_movement
	}

	fn compute_path_internal<TComputer>(
		&self,
		computer: &TComputer,
		transform: &GlobalTransform,
		end: Vec3,
	) -> VecDeque<Vec3>
	where
		TComputer: ComputePath,
	{
		let start = transform.translation();
		let Some(path) = computer.compute_path(start, end, self.radius) else {
			return VecDeque::from([]);
		};
		let mut path = path.peekable();

		match path.peek() {
			Some(first) if first == &start => VecDeque::from_iter(path.skip(1)),
			_ => VecDeque::from_iter(path),
		}
	}
}

#[cfg(test)]
mod test_new_path {
	use super::*;
	use common::tools::Units;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, marker::PhantomData};
	use testing::{NestedMocks, SingleThreadedApp, assert_no_panic};

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	#[derive(Component)]
	struct _GetComputer(Entity);

	impl GetProperty<Entity> for _GetComputer {
		fn get_property(&self) -> Entity {
			self.0
		}
	}

	#[derive(Component, NestedMocks)]
	struct _ComputePath {
		mock: Mock_ComputePath,
	}

	#[automock]
	impl ComputePath for _ComputePath {
		type TIter<'a>
			= Iter
		where
			Self: 'a;

		fn compute_path(&self, start: Vec3, end: Vec3, agent_radius: Units) -> Option<Iter> {
			self.mock.compute_path(start, end, agent_radius)
		}
	}

	#[derive(Clone)]
	pub struct Iter(VecDeque<Vec3>);

	impl Iterator for Iter {
		type Item = Vec3;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.pop_front()
		}
	}

	macro_rules! iter {
		() => {
			Iter(VecDeque::from([]))
		};
		($($values:expr),+ $(,)?) => {
			Iter(VecDeque::from([$($values),+]))
		};
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			MovementDefinition::compute_path::<_MoveMethod, _ComputePath, _GetComputer>,
		);

		app
	}

	mod path {
		use super::*;

		#[test]
		fn set_path() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![
						Vec3::splat(1.),
						Vec3::splat(2.),
						Vec3::splat(3.),
					]));
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([
						Vec3::splat(1.),
						Vec3::splat(2.),
						Vec3::splat(3.),
					])),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
		}

		#[test]
		fn set_no_path_path_when_cannot_be_computed() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(None);
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([])),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
		}

		#[test]
		fn set_path_ignoring_first_when_matching_translation() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![
						Vec3::splat(1.),
						Vec3::splat(2.),
						Vec3::splat(3.),
					]));
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
					GlobalTransform::from_translation(Vec3::splat(1.)),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Path(VecDeque::from([Vec3::splat(2.), Vec3::splat(3.)])),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
		}

		#[test]
		fn no_panic_if_path_len_zero() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![]));
				}))
				.id();
			app.world_mut().spawn((
				MovementDefinition {
					radius: Units::from(1.),
					..default()
				},
				Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
				GlobalTransform::default(),
				_GetComputer(computer),
			));

			assert_no_panic!(app.update());
		}

		#[test]
		fn remove_present_movement() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![
						Vec3::splat(1.),
						Vec3::splat(2.),
						Vec3::splat(3.),
					]));
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
					GlobalTransform::default(),
					Movement::<_MoveMethod>::to(Vec3::default()),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				None,
				app.world().entity(entity).get::<Movement::<_MoveMethod>>()
			);
		}

		#[test]
		fn compute_path_correctly() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path()
						.times(1)
						.with(
							eq(Vec3::new(1., 2., 3.)),
							eq(Vec3::new(4., 5., 6.)),
							eq(Units::from(42.)),
						)
						.return_const(None);
				}))
				.id();
			app.world_mut().spawn((
				MovementDefinition {
					radius: Units::from(42.),
					..default()
				},
				Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::new(4., 5., 6.)),
				GlobalTransform::from_xyz(1., 2., 3.),
				_GetComputer(computer),
			));

			app.update();
		}
	}
	mod wasd {
		use super::*;

		#[test]
		fn set_target_when_wasd() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().never().return_const(None);
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Dir3::NEG_Z),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection::<_MoveMethod> {
					mode: Mode::Direction(Some(Dir3::NEG_Z)),
					_m: PhantomData,
				}),
				app.world()
					.entity(entity)
					.get::<PathOrDirection<_MoveMethod>>()
			);
		}
	}

	mod update_control {
		use super::*;

		#[test]
		fn act_only_once() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().times(1).return_const(None);
				}))
				.id();
			app.world_mut().spawn((
				MovementDefinition {
					radius: Units::from(1.),
					..default()
				},
				Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
				GlobalTransform::default(),
				_GetComputer(computer),
			));

			app.update();
			app.update();
		}

		#[test]
		fn act_again_if_movement_changed() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().times(2).return_const(None);
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MovementDefinition {
						radius: Units::from(1.),
						..default()
					},
					Movement::<PathOrDirection<_MoveMethod>>::to(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.get_mut::<Movement<PathOrDirection<_MoveMethod>>>()
				.as_deref_mut();
			app.update();
		}
	}
}
