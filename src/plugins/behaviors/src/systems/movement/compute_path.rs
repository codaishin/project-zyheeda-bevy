use crate::{Movement, PathOrWasd, components::movement::path_or_wasd::Mode};
use bevy::prelude::*;
use common::{
	tools::collider_radius::ColliderRadius,
	traits::{
		accessors::get::{RefAs, RefInto, TryApplyOn},
		handles_path_finding::ComputePath,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::collections::VecDeque;

type MoveComponents<'a, TAgent, TMoveMethod, TGetComputer> = (
	Entity,
	&'a GlobalTransform,
	&'a TAgent,
	&'a Movement<PathOrWasd<TMoveMethod>>,
	&'a TGetComputer,
);
type ChangedMovement<TMoveMethod> = Changed<Movement<PathOrWasd<TMoveMethod>>>;

impl<T> MovementPath for T where T: Component + for<'a> RefInto<'a, ColliderRadius> + Sized {}

pub(crate) trait MovementPath:
	Component + for<'a> RefInto<'a, ColliderRadius> + Sized
{
	fn compute_path<TMoveMethod, TComputer, TGetComputer>(
		mut commands: ZyheedaCommands,
		movements: Query<
			MoveComponents<Self, TMoveMethod, TGetComputer>,
			ChangedMovement<TMoveMethod>,
		>,
		computers: Query<&TComputer>,
	) where
		TMoveMethod: ThreadSafe + Default,
		TComputer: Component + ComputePath,
		TGetComputer: Component + for<'a> RefInto<'a, Entity>,
	{
		for (entity, transform, agent, movement, get_computer) in &movements {
			let Ok(computer) = computers.get(get_computer.ref_as::<Entity>()) else {
				continue;
			};
			let move_component = new_movement(computer, transform, movement, agent);
			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(move_component);
				e.try_remove::<Movement<TMoveMethod>>();
			});
		}
	}
}

fn new_movement<TAgent, TMoveMethod, TComputer>(
	computer: &TComputer,
	transform: &GlobalTransform,
	movement: &Movement<PathOrWasd<TMoveMethod>>,
	agent: &TAgent,
) -> PathOrWasd<TMoveMethod>
where
	TAgent: for<'a> RefInto<'a, ColliderRadius>,
	TComputer: ComputePath,
	TMoveMethod: ThreadSafe,
{
	let mut new_movement = movement.new_movement();

	match &mut new_movement.mode {
		Mode::Wasd(target) => {
			*target = Some(movement.target);
		}
		Mode::Path(move_path) => {
			*move_path = compute_path(computer, transform, movement, agent);
		}
	}

	new_movement
}

fn compute_path<TAgent, TMoveMethod, TComputer>(
	computer: &TComputer,
	transform: &GlobalTransform,
	movement: &Movement<PathOrWasd<TMoveMethod>>,
	agent: &TAgent,
) -> VecDeque<Vec3>
where
	TAgent: for<'a> RefInto<'a, ColliderRadius>,
	TComputer: ComputePath,
	TMoveMethod: ThreadSafe,
{
	let start = transform.translation();
	let end = movement.target;
	let ColliderRadius(radius) = agent.ref_into();
	let Some(path) = computer.compute_path(start, end, radius) else {
		return VecDeque::from([]);
	};
	let path = match path.as_slice() {
		[first, rest @ ..] if first == &start => rest,
		path => path,
	};

	VecDeque::from_iter(path.iter().copied())
}

#[cfg(test)]
mod test_new_path {
	use super::*;
	use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, marker::PhantomData};
	use testing::{NestedMocks, SingleThreadedApp, assert_no_panic};

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	#[derive(Component)]
	struct _GetComputer(Entity);

	impl RefInto<'_, Entity> for _GetComputer {
		fn ref_into(&self) -> Entity {
			self.0
		}
	}

	#[derive(Component, NestedMocks)]
	struct _AgentMovement {
		mock: Mock_AgentMovement,
	}

	impl Default for _AgentMovement {
		fn default() -> Self {
			let mut mock = Mock_AgentMovement::new();
			mock.expect_ref_into()
				.return_const(ColliderRadius(Units::new(1.)));

			Self { mock }
		}
	}

	#[automock]
	impl RefInto<'_, ColliderRadius> for _AgentMovement {
		fn ref_into(&self) -> ColliderRadius {
			self.mock.ref_into()
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

		app.add_systems(
			Update,
			_AgentMovement::compute_path::<_MoveMethod, _ComputePath, _GetComputer>,
		);

		app
	}

	#[test]
	fn set_path() {
		let mut app = setup();
		let computer = app
			.world_mut()
			.spawn(_ComputePath::new().with_mock(|mock| {
				mock.expect_compute_path().return_const(Some(vec![
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.),
				]));
			}))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
				_GetComputer(computer),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.)
				])),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
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
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
				_GetComputer(computer),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([])),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
	}

	#[test]
	fn set_path_ignoring_first_when_matching_translation() {
		let mut app = setup();
		let computer = app
			.world_mut()
			.spawn(_ComputePath::new().with_mock(|mock| {
				mock.expect_compute_path().return_const(Some(vec![
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.),
				]));
			}))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::from_translation(Vec3::splat(1.)),
				_GetComputer(computer),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([Vec3::splat(2.), Vec3::splat(3.)])),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
	}

	#[test]
	fn no_panic_if_path_len_zero() {
		let mut app = setup();
		let computer = app
			.world_mut()
			.spawn(_ComputePath::new().with_mock(|mock| {
				mock.expect_compute_path().return_const(Some(vec![]));
			}))
			.id();
		app.world_mut().spawn((
			_AgentMovement::default(),
			Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
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
				mock.expect_compute_path().return_const(Some(vec![
					Vec3::splat(1.),
					Vec3::splat(2.),
					Vec3::splat(3.),
				]));
			}))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
				Movement::new(Vec3::default(), || _MoveMethod),
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
						eq(Units::new(42.)),
					)
					.return_const(None);
			}))
			.id();
		app.world_mut().spawn((
			_AgentMovement::new().with_mock(|mock| {
				mock.expect_ref_into()
					.return_const(ColliderRadius(Units::new(42.)));
			}),
			Movement::new(Vec3::new(4., 5., 6.), PathOrWasd::<_MoveMethod>::new_path),
			GlobalTransform::from_xyz(1., 2., 3.),
			_GetComputer(computer),
		));

		app.update();
	}

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
				_AgentMovement::default(),
				Movement::new(Vec3::new(1., 2., 3.), PathOrWasd::<_MoveMethod>::new_wasd),
				GlobalTransform::default(),
				_GetComputer(computer),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Wasd(Some(Vec3::new(1., 2., 3.))),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
	}

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
			_AgentMovement::default(),
			Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
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
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
				_GetComputer(computer),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Movement<PathOrWasd<_MoveMethod>>>()
			.as_deref_mut();
		app.update();
	}
}
