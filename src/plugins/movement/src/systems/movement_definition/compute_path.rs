use crate::{
	PathOrDirection,
	components::{movement::path_or_direction::Mode, movement_definition::MovementDefinition},
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, TryApplyOn},
		handles_map_generation::GroundPosition,
		handles_path_finding::ComputePath,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::VecDeque;

type MoveComponents<TGetComputer> = (
	Entity,
	&'static MovementDefinition,
	&'static GlobalTransform,
	&'static PathOrDirection,
	&'static TGetComputer,
);
type ChangedMovement = Changed<PathOrDirection>;

impl MovementDefinition {
	pub(crate) fn compute_path<TComputer, TGetComputer>(
		mut commands: ZyheedaCommands,
		movements: Query<MoveComponents<TGetComputer>, ChangedMovement>,
		computers: Query<&TComputer>,
	) where
		TComputer: Component + ComputePath,
		TGetComputer: Component + GetProperty<Entity>,
	{
		for (entity, definition, transform, path, get_computer) in &movements {
			let Ok(computer) = computers.get(get_computer.get_property()) else {
				continue;
			};
			let Mode::PathTarget(Some(target)) = path.mode else {
				continue;
			};
			let path = definition.compute_path_internal(computer, transform, target);

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(PathOrDirection::path(path));
			});
		}
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
		let mut path = path.map(|v| GroundPosition(v.with_y(start.y))).peekable();

		match path.peek() {
			Some(first) if **first == start => VecDeque::from_iter(path.skip(1).map(|p| *p)),
			_ => VecDeque::from_iter(path.map(|p| *p)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::Units;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::VecDeque;
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
	pub struct Iter(VecDeque<GroundPosition>);

	impl Iterator for Iter {
		type Item = GroundPosition;

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
			MovementDefinition::compute_path::<_ComputePath, _GetComputer>,
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
						GroundPosition(Vec3::splat(1.)),
						GroundPosition(Vec3::splat(2.)),
						GroundPosition(Vec3::splat(3.)),
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
					PathOrDirection::target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection {
					mode: Mode::Path(VecDeque::from([
						Vec3::new(1., 0., 1.),
						Vec3::new(2., 0., 2.),
						Vec3::new(3., 0., 3.),
					])),
				}),
				app.world().entity(entity).get::<PathOrDirection>()
			);
		}

		#[test]
		fn set_path_with_current_height() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![
						GroundPosition(Vec3::splat(1.)),
						GroundPosition(Vec3::splat(2.)),
						GroundPosition(Vec3::splat(3.)),
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
					PathOrDirection::target(Vec3::default()),
					GlobalTransform::from_xyz(0., 11., 0.),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection {
					mode: Mode::Path(VecDeque::from([
						Vec3::new(1., 11., 1.),
						Vec3::new(2., 11., 2.),
						Vec3::new(3., 11., 3.),
					])),
				}),
				app.world().entity(entity).get::<PathOrDirection>()
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
					PathOrDirection::target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection {
					mode: Mode::Path(VecDeque::from([])),
				}),
				app.world().entity(entity).get::<PathOrDirection>()
			);
		}

		#[test]
		fn set_path_ignoring_first_when_matching_translation() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().return_const(Some(iter![
						GroundPosition(Vec3::splat(1.)),
						GroundPosition(Vec3::splat(2.)),
						GroundPosition(Vec3::splat(3.)),
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
					PathOrDirection::target(Vec3::default()),
					GlobalTransform::from_translation(Vec3::ONE),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection {
					mode: Mode::Path(VecDeque::from([
						Vec3::new(2., 1., 2.),
						Vec3::new(3., 1., 3.),
					])),
				}),
				app.world().entity(entity).get::<PathOrDirection>()
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
				PathOrDirection::target(Vec3::default()),
				GlobalTransform::default(),
				_GetComputer(computer),
			));

			assert_no_panic!(app.update());
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
				PathOrDirection::target(Vec3::new(4., 5., 6.)),
				GlobalTransform::from_xyz(1., 2., 3.),
				_GetComputer(computer),
			));

			app.update();
		}
	}
	mod direction {
		use super::*;

		#[test]
		fn set_target_when_direction() {
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
					PathOrDirection::direction(Dir3::NEG_Z),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&PathOrDirection::direction(Dir3::NEG_Z)),
				app.world().entity(entity).get::<PathOrDirection>()
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
				PathOrDirection::target(Vec3::default()),
				GlobalTransform::default(),
				_GetComputer(computer),
			));

			app.update();
			app.update();
		}

		#[test]
		fn act_again_if_path_changed() {
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
					PathOrDirection::target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();
			app.world_mut()
				.entity_mut(entity)
				.insert(PathOrDirection::target(Vec3::default()));
			app.update();
		}
	}
}
