use crate::{
	components::{
		map::{MapObjectType, objects::MapObjectOf},
		map_agents::GridAgent,
		spawned::Spawned,
		spawner::Spawner,
		spawner_active::SpawnerActive,
	},
	resources::agents::prefab::PrefabRegister,
};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		handles_map_generation::PrefabType,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl<T> Spawner<T>
where
	T: PrefabType<TTranslation: From<Vec3>> + Copy + ThreadSafe + Into<MapObjectType>,
{
	pub(crate) fn execute(
		mut commands: ZyheedaCommands,
		spawners: Query<(Entity, &Self, &GlobalTransform, &MapObjectOf), With<SpawnerActive>>,
		agent_prefabs: Res<PrefabRegister<T>>,
	) {
		for (entity, Self(agent_type), transform, MapObjectOf(map)) in spawners {
			let agent = commands.spawn((
				*transform,
				GridAgent,
				MapObjectOf(*map),
				Spawned::from(*agent_type),
			));
			agent_prefabs.apply(
				ZyheedaEntityCommands::from(agent),
				T::TTranslation::from(transform.translation()),
				*agent_type,
			);

			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SpawnerActive>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		map::{MapObjectType, objects::MapObjectOf},
		spawned::Spawned,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::{
			handles_enemies::EnemyType,
			handles_map_generation::{AgentType, GroundPosition},
		},
	};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Debug, PartialEq)]
	struct _Agent {
		ground_position: Vec3,
		agent_type: AgentType,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(PrefabRegister(
			|mut e, GroundPosition(ground_position), agent_type| {
				e.try_insert(_Agent {
					ground_position,
					agent_type,
				});
			},
		));
		app.add_systems(Update, Spawner::<AgentType>::execute);

		app
	}

	#[test]
	fn spawn_agent() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::from_xyz(1., 2., 3.),
		));
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Enemy(EnemyType::VoidSphere)),
			GlobalTransform::from_xyz(4., 5., 6.),
		));

		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&_Agent {
					ground_position: Vec3::new(1., 2., 3.),
					agent_type: AgentType::Player
				},
				&_Agent {
					ground_position: Vec3::new(4., 5., 6.),
					agent_type: AgentType::Enemy(EnemyType::VoidSphere)
				},
			],
			agents
		);
	}

	#[test]
	fn spawn_agent_with_grid_agent_marker() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::from_xyz(1., 2., 3.),
		));
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Enemy(EnemyType::VoidSphere)),
			GlobalTransform::from_xyz(4., 5., 6.),
		));

		app.update();

		let mut agents = app.world_mut().query::<&GridAgent>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!([&GridAgent, &GridAgent], agents);
	}

	#[test]
	fn spawn_agent_with_type_marker() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::from_xyz(1., 2., 3.),
		));
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Enemy(EnemyType::VoidSphere)),
			GlobalTransform::from_xyz(4., 5., 6.),
		));

		app.update();

		let mut agents = app.world_mut().query::<&Spawned>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&Spawned(MapObjectType::Agent(AgentType::Player)),
				&Spawned(MapObjectType::Agent(AgentType::Enemy(
					EnemyType::VoidSphere
				)))
			],
			agents
		);
	}

	#[test]
	fn apply_transform() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)),
		));
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Enemy(EnemyType::VoidSphere)),
			GlobalTransform::from(Transform::from_xyz(4., 5., 6.).looking_to(Dir3::Z, Dir3::Y)),
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&GlobalTransform, Without<Spawner<AgentType>>>();
		let agents = assert_count!(2, agents.iter(app.world()));
		assert_eq!(
			[
				&GlobalTransform::from(
					Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)
				),
				&GlobalTransform::from(
					Transform::from_xyz(4., 5., 6.).looking_to(Dir3::Z, Dir3::Y)
				),
			],
			agents,
		);
	}

	#[test]
	fn set_map_reference() {
		let mut app = setup();
		let map_persistent = PersistentEntity::default();
		let map = app.world_mut().spawn(map_persistent).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&MapObjectOf, With<_Agent>>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!([&MapObjectOf(map)], agents);
	}

	#[test]
	fn inactivate() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		let entity = app
			.world_mut()
			.spawn((
				MapObjectOf(map),
				Spawner(AgentType::Player),
				GlobalTransform::default(),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SpawnerActive>());
	}

	#[test]
	fn do_nothing_if_spawner_inactive() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		let mut entity = app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(AgentType::Player),
			GlobalTransform::default(),
		));
		entity.remove::<SpawnerActive>();

		app.update();

		let mut agents = app.world_mut().query::<&_Agent>();
		assert_count!(0, agents.iter(app.world()));
	}
}
