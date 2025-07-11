use crate::{Movement, PathOrWasd, components::movement::path_or_wasd::Mode};
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	tools::collider_radius::ColliderRadius,
	traits::{
		accessors::get::{GetField, GetRef, Getter},
		handles_map_generation::EntityMapFiltered,
		handles_path_finding::ComputePath,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};
use std::{collections::VecDeque, marker::PhantomData};

impl<T> MovementPath for T where T: Component + Getter<ColliderRadius> + Sized {}

pub(crate) trait MovementPath: Component + Getter<ColliderRadius> + Sized {
	fn compute_path<TMoveMethod, TComputer, TGetComputer>()
	-> ComputePathSystemBuilder<Self, TMoveMethod, TComputer, TGetComputer> {
		ComputePathSystemBuilder(PhantomData)
	}
}

pub(crate) struct ComputePathSystemBuilder<TAgent, TMoveMethod, TComputer, TGetComputer>(
	PhantomData<(TAgent, TMoveMethod, TComputer, TGetComputer)>,
);

type Components<'a, TMoveMethod, TAgent> = (
	Entity,
	&'a GlobalTransform,
	&'a TAgent,
	&'a Movement<PathOrWasd<TMoveMethod>>,
);

impl<TAgent, TMoveMethod, TComputer, TGetComputer>
	ComputePathSystemBuilder<TAgent, TMoveMethod, TComputer, TGetComputer>
where
	TAgent: Component + Getter<ColliderRadius>,
	TMoveMethod: ThreadSafe + Default,
	TComputer: Component + ComputePath,
	TGetComputer: Getter<Entity>,
{
	#[allow(clippy::type_complexity)]
	pub(crate) fn system<TFilter>(
		self,
	) -> impl Fn(
		In<EntityMapFiltered<TGetComputer, TFilter>>,
		Commands,
		Query<Components<TMoveMethod, TAgent>, TFilter>,
		Query<&TComputer>,
	)
	where
		TFilter: QueryFilter,
	{
		|In(mapping), mut commands, mut movements, computers| {
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
	use mockall::{automock, mock, predicate::eq};
	use std::{collections::VecDeque, marker::PhantomData};
	use testing::{Mock, NestedMocks, SingleThreadedApp, simple_init};

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

	fn test_system<TGetComputer, TFilter>()
	-> impl IntoSystem<In<EntityMapFiltered<TGetComputer, TFilter>>, (), ()>
	where
		TGetComputer: Getter<Entity> + 'static,
		TFilter: QueryFilter + 'static,
	{
		let builder = _AgentMovement::compute_path::<_MoveMethod, _ComputePath, TGetComputer>();

		IntoSystem::into_system(builder.system::<TFilter>())
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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
			test_system::<_GetComputer, ()>(),
			EntityMapFiltered::from([(entity, _GetComputer(computer))]),
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

	#[test]
	fn apply_filter() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct Ignore;

		mock! {
			_GetComputer {}
			impl Getter<Entity> for _GetComputer {
				fn get(&self) -> Entity;
			}
		}

		simple_init!(Mock_GetComputer);

		let mut app = setup();
		let computer = app.world_mut().spawn_empty().id();
		let entity = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
			))
			.id();
		let ignore = app
			.world_mut()
			.spawn((
				_AgentMovement::default(),
				Movement::new(Vec3::default(), PathOrWasd::<_MoveMethod>::new_path),
				GlobalTransform::default(),
				Ignore,
			))
			.id();

		app.world_mut().run_system_once_with(
			test_system::<Mock_GetComputer, Without<Ignore>>(),
			EntityMapFiltered::from([
				(
					entity,
					Mock_GetComputer::new_mock(|mock| {
						mock.expect_get().times(1).return_const(computer);
					}),
				),
				(
					ignore,
					Mock_GetComputer::new_mock(|mock| {
						mock.expect_get().never();
					}),
				),
			]),
		)
	}
}
