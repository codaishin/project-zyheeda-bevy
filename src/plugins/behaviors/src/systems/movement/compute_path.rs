use crate::{Movement, PathOrWasd, components::movement::path_or_wasd::Mode};
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
use std::collections::{HashMap, VecDeque};

impl<T> MovementPath for T where T: Component + Getter<ColliderRadius> + Sized {}

type Components<'a, TMoveMethod, TAgent> = (
	Entity,
	&'a GlobalTransform,
	&'a TAgent,
	&'a Movement<PathOrWasd<TMoveMethod>>,
);

pub(crate) trait MovementPath: Component + Getter<ColliderRadius> + Sized {
	fn compute_path<TMoveMethod, TComputer, TGetComputer>(
		In(mapping): In<HashMap<Entity, TGetComputer>>,
		mut commands: Commands,
		mut movements: Query<Components<TMoveMethod, Self>>,
		computers: Query<&TComputer>,
	) where
		TMoveMethod: ThreadSafe + Default,
		TComputer: Component + ComputePath,
		TGetComputer: Getter<Entity>,
	{
		if movements.is_empty() {
			return;
		}

		for (entity, transform, agent, movement) in &mut movements {
			let Some(computer_ref) = mapping.get(&entity) else {
				continue;
			};
			let Ok(computer) = computers.get(Entity::get_field(computer_ref)) else {
				continue;
			};
			let move_component = new_movement(computer, transform, movement, agent);
			commands.try_insert_on(entity, move_component);
			commands.try_remove_from::<Movement<TMoveMethod>>(entity);
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
	TAgent: Getter<ColliderRadius>,
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
	TAgent: Getter<ColliderRadius>,
	TComputer: ComputePath,
	TMoveMethod: ThreadSafe,
{
	let start = transform.translation();
	let end = movement.target;
	let ColliderRadius(radius) = ColliderRadius::get_field(agent);
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{tools::Units, traits::clamp_zero_positive::ClampZeroPositive};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::VecDeque, marker::PhantomData};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Default)]
	struct _MoveMethod;

	struct _GetComputer(Entity);

	impl Getter<Entity> for _GetComputer {
		fn get(&self) -> Entity {
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

	fn test_system() -> impl IntoSystem<In<HashMap<Entity, _GetComputer>>, (), ()> {
		IntoSystem::into_system(
			_AgentMovement::compute_path::<_MoveMethod, _ComputePath, _GetComputer>,
		)
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn set_path() -> Result<(), RunSystemError> {
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
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)?;

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
		Ok(())
	}

	#[test]
	fn set_no_path_path_when_cannot_be_computed() -> Result<(), RunSystemError> {
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
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)?;

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([])),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
		Ok(())
	}

	#[test]
	fn set_path_ignoring_first_when_matching_translation() -> Result<(), RunSystemError> {
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
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)?;

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Path(VecDeque::from([Vec3::splat(2.), Vec3::splat(3.)])),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
		Ok(())
	}

	#[test]
	fn no_panic_if_path_len_zero() -> Result<(), RunSystemError> {
		let mut app = setup();
		let computer = app
			.world_mut()
			.spawn(_ComputePath::new().with_mock(|mock| {
				mock.expect_compute_path().return_const(Some(vec![]));
			}))
			.id();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)
	}

	#[test]
	fn remove_present_movement() -> Result<(), RunSystemError> {
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
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)?;

		assert_eq!(
			None,
			app.world().entity(entity).get::<Movement::<_MoveMethod>>()
		);
		Ok(())
	}

	#[test]
	fn compute_path_correctly() -> Result<(), RunSystemError> {
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
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::new().with_mock(|mock| {
					mock.expect_get()
						.return_const(ColliderRadius(Units::new(42.)));
				}),
				Movement::new(Vec3::new(4., 5., 6.), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::from_xyz(1., 2., 3.),
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)
	}

	#[test]
	fn set_target_when_wasd() -> Result<(), RunSystemError> {
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
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system(),
			HashMap::from([(entity, _GetComputer(computer))]),
		)?;

		assert_eq!(
			Some(&PathOrWasd::<_MoveMethod> {
				mode: Mode::Wasd(Some(Vec3::new(1., 2., 3.))),
				_m: PhantomData,
			}),
			app.world().entity(entity).get::<PathOrWasd<_MoveMethod>>()
		);
		Ok(())
	}
}
