use crate::{
	components::{
		agent_source::AgentSource,
		map::{MapObjectSource, objects::MapObjectOf},
		map_agents::GridAgent,
		spawned_from::SpawnedFrom,
		spawner::Spawner,
		spawner_active::SpawnerActive,
	},
	resources::agents::prefab::PrefabRegister,
};
use bevy::{gltf::GltfMeshName, prelude::*};
use common::prelude::*;

impl<T> Spawner<T>
where
	T: PrefabType<TTransform: From<GlobalTransform>> + Copy + ThreadSafe,
	Self: SpawnedExtra<TSource = T>,
{
	pub(crate) fn execute(
		mut commands: ZyheedaCommands,
		spawners: Query<
			(Entity, &Self, &GlobalTransform, &MapObjectOf, &GltfMeshName),
			With<SpawnerActive>,
		>,
		prefab_register: Res<PrefabRegister<T>>,
	) {
		for (entity, Self(source), transform, MapObjectOf(map), GltfMeshName(name)) in spawners {
			let spawned = commands.spawn((
				*transform,
				GridAgent,
				MapObjectOf(*map),
				SpawnedFrom(MapObjectSource(name.clone())),
				Self::spawned_extra(*source),
			));

			prefab_register.apply(
				ZyheedaEntityCommands::from(spawned),
				T::TTransform::from(*transform),
				*source,
			);

			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SpawnerActive>();
			});
		}
	}
}

pub(crate) trait SpawnedExtra {
	type TBundle: Bundle;
	type TSource;

	fn spawned_extra(source: Self::TSource) -> Self::TBundle;
}

impl SpawnedExtra for Spawner<AgentType> {
	type TBundle = AgentSource;
	type TSource = AgentType;

	fn spawned_extra(source: Self::TSource) -> Self::TBundle {
		AgentSource(source)
	}
}

impl SpawnedExtra for Spawner<InteractiveType> {
	type TBundle = ();
	type TSource = InteractiveType;

	fn spawned_extra(_: Self::TSource) -> Self::TBundle {}
}

impl SpawnedExtra for Spawner<LightType> {
	type TBundle = ();
	type TSource = LightType;

	fn spawned_extra(_: Self::TSource) -> Self::TBundle {}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{map::objects::MapObjectOf, spawned_from::SpawnedFrom};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _T;

	impl PrefabType for _T {
		type TTransform = GlobalTransform;
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Extra(_T);

	impl SpawnedExtra for Spawner<_T> {
		type TBundle = _Extra;
		type TSource = _T;

		fn spawned_extra(source: Self::TSource) -> Self::TBundle {
			_Extra(source)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Spawned {
		transform: GlobalTransform,
		source: _T,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(PrefabRegister(|mut e, transform, source| {
			e.try_insert(_Spawned { transform, source });
		}));
		app.add_systems(Update, Spawner::<_T>::execute);

		app
	}

	#[test]
	fn spawn() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(_T),
			GlobalTransform::from_xyz(1., 2., 3.),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app.world_mut().query::<&_Spawned>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!(
			[&_Spawned {
				transform: GlobalTransform::from_xyz(1., 2., 3.),
				source: _T
			},],
			agents
		);
	}

	#[test]
	fn spawn_with_grid_agent_marker() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(_T),
			GlobalTransform::from_xyz(1., 2., 3.),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app.world_mut().query::<&GridAgent>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!([&GridAgent], agents);
	}

	#[test]
	fn spawn_with_type_marker() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(_T),
			GlobalTransform::from_xyz(1., 2., 3.),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app.world_mut().query::<&SpawnedFrom>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!([&SpawnedFrom(MapObjectSource(String::from("a")))], agents);
	}

	#[test]
	fn apply_transform() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(_T),
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&GlobalTransform, Without<Spawner<_T>>>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!(
			[&GlobalTransform::from(
				Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y)
			)],
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
			Spawner(_T),
			GlobalTransform::from_xyz(1., 2., 3.),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app
			.world_mut()
			.query_filtered::<&MapObjectOf, With<_Spawned>>();
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
				Spawner(_T),
				GlobalTransform::default(),
				GltfMeshName(String::from("a")),
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
			Spawner(_T),
			GlobalTransform::default(),
			GltfMeshName(String::from("a")),
		));
		entity.remove::<SpawnerActive>();

		app.update();

		let mut agents = app.world_mut().query::<&_Spawned>();
		assert_count!(0, agents.iter(app.world()));
	}

	#[test]
	fn spawn_with_extra() {
		let mut app = setup();
		let map = app.world_mut().spawn(PersistentEntity::default()).id();
		app.world_mut().spawn((
			MapObjectOf(map),
			Spawner(_T),
			GlobalTransform::from_xyz(1., 2., 3.),
			GltfMeshName(String::from("a")),
		));

		app.update();

		let mut agents = app.world_mut().query_filtered::<&_Extra, With<_Spawned>>();
		let agents = assert_count!(1, agents.iter(app.world()));
		assert_eq!([&_Extra(_T)], agents);
	}
}
