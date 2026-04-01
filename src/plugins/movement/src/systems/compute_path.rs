use crate::components::{config::Config, movement::Movement};
use bevy::prelude::*;
use common::traits::{
	accessors::get::View,
	handles_map_generation::GroundPosition,
	handles_path_finding::ComputePath,
};
use std::collections::VecDeque;

type MoveComponents<TGetComputer> = (
	&'static Config,
	&'static GlobalTransform,
	&'static mut Movement,
	&'static TGetComputer,
);

impl Movement {
	pub(crate) fn compute_path<TComputer, TGetComputer>(
		movements: Query<MoveComponents<TGetComputer>>,
		computers: Query<&TComputer>,
	) where
		TComputer: Component + ComputePath,
		TGetComputer: Component + View<Entity>,
	{
		for (config, transform, mut path, get_computer) in movements {
			let Movement::Target(target) = path.as_ref() else {
				continue;
			};
			let Ok(computer) = computers.get(get_computer.view()) else {
				continue;
			};

			*path = Movement::Path(compute_path(computer, transform, *target, config));
		}
	}
}

fn compute_path<TComputer>(
	computer: &TComputer,
	transform: &GlobalTransform,
	end: Vec3,
	Config {
		required_clearance,
		ground_offset,
		..
	}: &Config,
) -> VecDeque<Vec3>
where
	TComputer: ComputePath,
{
	let start = transform.translation();
	let Some(path) = computer.compute_path(start, end, *required_clearance) else {
		return VecDeque::from([]);
	};
	let mut path = path
		.map(|GroundPosition(v)| v.with_y(v.y + **ground_offset))
		.peekable();

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

		app.add_systems(Update, Movement::compute_path::<_ComputePath, _GetComputer>);

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
					Config::default(),
					Movement::Target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::path([
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.),
				])),
				app.world().entity(entity).get::<Movement>()
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
					Config {
						ground_offset: Units::from(2.),
						..default()
					},
					Movement::Target(Vec3::default()),
					GlobalTransform::from_xyz(0., 11., 0.),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::path([
					Vec3::splat(1.) + Vec3::new(0., 2., 0.),
					Vec3::splat(2.) + Vec3::new(0., 2., 0.),
					Vec3::splat(3.) + Vec3::new(0., 2., 0.),
				])),
				app.world().entity(entity).get::<Movement>()
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
					Config::default(),
					Movement::Target(Vec3::default()),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::path([])),
				app.world().entity(entity).get::<Movement>()
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
					Config::default(),
					Movement::Target(Vec3::default()),
					GlobalTransform::from_translation(Vec3::new(1., 0., 1.)),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::path([Vec3::splat(2.), Vec3::splat(3.)])),
				app.world().entity(entity).get::<Movement>()
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
				Config {
					required_clearance: Units::from_u8(1),
					..default()
				},
				Movement::Target(Vec3::default()),
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
				Config {
					required_clearance: Units::from_u8(42),
					..default()
				},
				Movement::Target(Vec3::new(4., 5., 6.)),
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
					Config::default(),
					Movement::Direction(Dir3::NEG_Z),
					GlobalTransform::default(),
					_GetComputer(computer),
				))
				.id();

			app.update();

			assert_eq!(
				Some(&Movement::Direction(Dir3::NEG_Z)),
				app.world().entity(entity).get::<Movement>()
			);
		}
	}
}
