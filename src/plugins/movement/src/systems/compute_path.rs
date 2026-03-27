use crate::{MovementPath, components::movement_path::Mode};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	traits::{
		accessors::get::{TryApplyOn, View, ViewOf},
		handles_map_generation::GroundPosition,
		handles_movement::{GroundOffset, RequiredClearance},
		handles_path_finding::ComputePath,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::VecDeque;

type MoveComponents<TGetComputer, TConfig> = (
	Entity,
	&'static TConfig,
	&'static GlobalTransform,
	&'static MovementPath,
	&'static TGetComputer,
);

impl<T> ComputePathSystem for T where T: QueryFilter {}

pub(crate) trait ComputePathSystem: QueryFilter + Sized {
	fn compute<TComputer, TGetComputer, TConfig>(
		mut commands: ZyheedaCommands,
		movements: Query<MoveComponents<TGetComputer, TConfig>, Self>,
		computers: Query<&TComputer>,
	) where
		TComputer: Component + ComputePath,
		TGetComputer: Component + View<Entity>,
		TConfig: Component + View<RequiredClearance> + View<GroundOffset>,
	{
		for (entity, config, transform, path, get_computer) in &movements {
			let Ok(computer) = computers.get(get_computer.view()) else {
				continue;
			};
			let Mode::PathTarget(Some(target)) = path.0 else {
				continue;
			};
			let path = compute_path(computer, transform, target, config);

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(MovementPath::path(path));
			});
		}
	}
}

fn compute_path<TComputer, TConfig>(
	computer: &TComputer,
	transform: &GlobalTransform,
	end: Vec3,
	config: &TConfig,
) -> VecDeque<Vec3>
where
	TComputer: ComputePath,
	TConfig: View<RequiredClearance> + View<GroundOffset>,
{
	let start = transform.translation();
	let required_clearance = config.view_of::<RequiredClearance>();
	let ground_offset = config.view_of::<GroundOffset>();
	let Some(path) = computer.compute_path(start, end, required_clearance) else {
		return VecDeque::from([]);
	};
	let mut path = path.map(|GroundPosition(v)| v + ground_offset).peekable();

	match path.peek() {
		Some(first) if first.xz() == start.xz() => VecDeque::from_iter(path.skip(1)),
		_ => VecDeque::from_iter(path),
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

	#[derive(Component, Default)]
	struct _Config {
		required_clearance: Units,
		ground_offset: Vec3,
	}

	impl View<RequiredClearance> for _Config {
		fn view(&self) -> Units {
			self.required_clearance
		}
	}

	impl View<GroundOffset> for _Config {
		fn view(&self) -> Vec3 {
			self.ground_offset
		}
	}

	#[derive(Component)]
	struct _ExecComputation;

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	#[derive(Component)]
	struct _GetComputer(Entity);

	impl View<Entity> for _GetComputer {
		fn view(&self) -> Entity {
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
			With::<_ExecComputation>::compute::<_ComputePath, _GetComputer, _Config>,
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
					_ExecComputation,
					_Config::default(),
					MovementPath::target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementPath::path([
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.),
				])),
				app.world().entity(entity).get::<MovementPath>()
			);
		}

		#[test]
		fn set_path_with_ground_offset() {
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
					_ExecComputation,
					_Config {
						ground_offset: Vec3::new(1., 2., 3.),
						..default()
					},
					MovementPath::target(Vec3::default()),
					GlobalTransform::from_xyz(0., 11., 0.),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementPath::path([
					Vec3::splat(1.) + Vec3::new(1., 2., 3.),
					Vec3::splat(2.) + Vec3::new(1., 2., 3.),
					Vec3::splat(3.) + Vec3::new(1., 2., 3.),
				])),
				app.world().entity(entity).get::<MovementPath>()
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
					_ExecComputation,
					_Config::default(),
					MovementPath::target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementPath::path([])),
				app.world().entity(entity).get::<MovementPath>()
			);
		}

		#[test]
		fn set_path_ignoring_first_when_matching_translation_horizontally() {
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
					_ExecComputation,
					_Config::default(),
					MovementPath::target(Vec3::default()),
					GlobalTransform::from_translation(Vec3::new(1., 0., 1.)),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementPath::path([Vec3::splat(2.), Vec3::splat(3.)])),
				app.world().entity(entity).get::<MovementPath>()
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
				_ExecComputation,
				_Config {
					required_clearance: Units::from_u8(1),
					..default()
				},
				MovementPath::target(Vec3::default()),
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
				_ExecComputation,
				_Config {
					required_clearance: Units::from_u8(42),
					..default()
				},
				MovementPath::target(Vec3::new(4., 5., 6.)),
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
					_ExecComputation,
					_Config::default(),
					MovementPath::direction(Dir3::NEG_Z),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&MovementPath::direction(Dir3::NEG_Z)),
				app.world().entity(entity).get::<MovementPath>()
			);
		}
	}

	mod update_control {
		use super::*;

		#[test]
		fn do_nothing_when_query_filter_does_not_apply() {
			let mut app = setup();
			let computer = app
				.world_mut()
				.spawn(_ComputePath::new().with_mock(|mock| {
					mock.expect_compute_path().never().return_const(None);
				}))
				.id();
			app.world_mut().spawn((
				// NO `_ExecComputation``
				_Config {
					required_clearance: Units::from_u8(1),
					..default()
				},
				MovementPath::target(Vec3::default()),
				GlobalTransform::default(),
				_GetComputer(computer),
			));

			app.update();
		}
	}
}
