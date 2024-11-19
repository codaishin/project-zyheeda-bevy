use crate::components::SpawnAfterInstantiation;
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{
		cache::{GetOrCreateAsset, GetOrCreateAssetFactory},
		prefab::Prefab,
	},
};
use std::any::TypeId;

type SMat = StandardMaterial;
type GetHandleFn<'a, TAsset> =
	&'a mut dyn FnMut(TypeId, &mut dyn FnMut() -> TAsset) -> Handle<TAsset>;

struct GetHandlesFns<'a> {
	mesh: GetHandleFn<'a, Mesh>,
	material: GetHandleFn<'a, SMat>,
}

impl<'a> GetOrCreateAsset<TypeId, Mesh> for GetHandlesFns<'a> {
	fn get_or_create(&mut self, key: TypeId, mut create: impl FnMut() -> Mesh) -> Handle<Mesh> {
		(self.mesh)(key, &mut create)
	}
}

impl<'a> GetOrCreateAsset<TypeId, SMat> for GetHandlesFns<'a> {
	fn get_or_create(&mut self, key: TypeId, mut create: impl FnMut() -> SMat) -> Handle<SMat> {
		(self.material)(key, &mut create)
	}
}

pub fn instantiate<TAgent, TMeshAssets, TSMatAssets, TMeshStorage, TSMatStorage, TFactory>(
	mut commands: Commands,
	meshes: ResMut<TMeshAssets>,
	smats: ResMut<TSMatAssets>,
	mesh_storage: ResMut<TMeshStorage>,
	smat_storage: ResMut<TSMatStorage>,
	agents: Query<(Entity, &TAgent), Added<TAgent>>,
) -> Vec<Result<(), Error>>
where
	TAgent: Component + Prefab,
	TMeshAssets: Resource,
	TSMatAssets: Resource,
	TMeshStorage: Resource,
	TSMatStorage: Resource,
	TFactory: GetOrCreateAssetFactory<TMeshAssets, Mesh, TMeshStorage, TypeId>
		+ GetOrCreateAssetFactory<TSMatAssets, SMat, TSMatStorage, TypeId>
		+ 'static,
{
	let mesh_cache = &mut TFactory::create_from(meshes, mesh_storage);
	let smat_cache = &mut TFactory::create_from(smats, smat_storage);

	let instantiate = |(entity, agent): (Entity, &TAgent)| {
		let Some(mut entity) = commands.get_entity(entity) else {
			return Err(Error {
				msg: format!("Cannot instantiate prefab, because {entity:?} does not exist",),
				lvl: Level::Error,
			});
		};
		agent.instantiate_on::<SpawnAfterInstantiation>(
			&mut entity,
			GetHandlesFns {
				mesh: &mut |type_id: TypeId, mesh: &mut dyn FnMut() -> Mesh| {
					mesh_cache.get_or_create(type_id, mesh)
				},
				material: &mut |type_id: TypeId, material: &mut dyn FnMut() -> SMat| {
					smat_cache.get_or_create(type_id, material)
				},
			},
		)
	};

	agents.iter().map(instantiate).collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		color::Color,
		ecs::system::{EntityCommands, IntoSystem, RunSystemOnce},
		math::primitives::Sphere,
		prelude::default,
	};
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
		traits::{
			cache::GetOrCreateTypeAsset,
			prefab::{AfterInstantiation, GetOrCreateAssets},
		},
	};
	use std::marker::PhantomData;
	use uuid::Uuid;

	#[derive(Component)]
	struct _Agent;

	impl _Agent {
		fn mesh() -> Mesh {
			Mesh::from(Sphere { radius: 11. })
		}

		fn material() -> SMat {
			SMat::from(Color::srgb(0., 0., 1.))
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Child;

	#[derive(Component)]
	struct _Result<TAsset: Asset>(Handle<TAsset>);

	impl Prefab for _Agent {
		fn instantiate_on<TAfterInstantiation>(
			&self,
			entity: &mut EntityCommands,
			mut assets: impl GetOrCreateAssets,
		) -> Result<(), Error>
		where
			TAfterInstantiation: AfterInstantiation,
		{
			entity.try_insert((
				_Result(assets.get_or_create_for::<_Agent>(_Agent::mesh)),
				_Result(assets.get_or_create_for::<_Agent>(_Agent::material)),
				TAfterInstantiation::spawn(|parent| {
					parent.spawn(_Child);
				}),
			));
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithInstantiationError;

	impl Prefab for _AgentWithInstantiationError {
		fn instantiate_on<TAfterInstantiation>(
			&self,
			_: &mut EntityCommands,
			_: impl GetOrCreateAssets,
		) -> Result<(), Error> {
			Err(Error {
				msg: "AAA".to_owned(),
				lvl: Level::Warning,
			})
		}
	}

	#[derive(Resource)]
	struct _Assets<TAsset>(PhantomData<TAsset>);

	impl<T> Default for _Assets<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}

	#[derive(Resource)]
	struct _Storage<TAsset>(PhantomData<TAsset>);

	impl<T> Default for _Storage<T> {
		fn default() -> Self {
			Self(PhantomData)
		}
	}

	struct _Cache<TAsset: Asset> {
		args: Vec<(TypeId, TAsset)>,
		returns: Handle<TAsset>,
		assert: Option<fn(&_Cache<TAsset>)>,
	}

	impl<T: Asset> Default for _Cache<T> {
		fn default() -> Self {
			Self {
				args: default(),
				returns: default(),
				assert: None,
			}
		}
	}

	impl<TAsset: Asset> GetOrCreateAsset<TypeId, TAsset> for _Cache<TAsset> {
		fn get_or_create(
			&mut self,
			key: TypeId,
			create: impl FnOnce() -> TAsset,
		) -> Handle<TAsset> {
			self.args.push((key, create()));
			if let Some(assert) = self.assert {
				assert(self);
			}
			self.returns.clone()
		}
	}

	fn setup<TAgent: Component + Prefab, TCombine>() -> (App, Entity)
	where
		for<'a> TCombine: GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId>
			+ GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId>
			+ 'static,
	{
		let mut app = App::new();
		let logger = app.world_mut().spawn_empty().id();
		let instantiate_system = instantiate::<
			TAgent,
			_Assets<Mesh>,
			_Assets<SMat>,
			_Storage<Mesh>,
			_Storage<SMat>,
			TCombine,
		>;
		app.init_resource::<_Assets<Mesh>>();
		app.init_resource::<_Assets<SMat>>();
		app.init_resource::<_Storage<Mesh>>();
		app.init_resource::<_Storage<SMat>>();
		app.add_systems(
			Update,
			instantiate_system.pipe(fake_log_error_lazy_many(logger)),
		);

		(app, logger)
	}

	const fn new_handle<T: Asset>(raw_uuid: u128) -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(raw_uuid),
		})
	}

	#[test]
	fn instantiate_mesh() {
		static HANDLE: Handle<Mesh> = new_handle(0xe1cdbce7_19f4_4b10_8bf6_80e5ca26f266);

		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache {
					returns: HANDLE.clone(),
					..default()
				}
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache::default()
			}
		}

		let (mut app, ..) = setup::<_Agent, _Factory>();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		let agent = app.world().entity(agent);
		let result = agent.get::<_Result<Mesh>>();

		assert_eq!(Some(HANDLE.clone()), result.map(|r| r.0.clone()));
	}

	#[test]
	fn instantiate_material() {
		static HANDLE: Handle<SMat> = new_handle(0xe1cdbce7_19f4_4b10_8bf6_80e5ca26f266);

		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache::default()
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache {
					returns: HANDLE.clone(),
					..default()
				}
			}
		}

		let (mut app, ..) = setup::<_Agent, _Factory>();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();

		let agent = app.world().entity(agent);
		let result = agent.get::<_Result<SMat>>();

		assert_eq!(Some(HANDLE.clone()), result.map(|r| r.0.clone()));
	}

	fn children(app: &App, entity: Entity) -> impl Iterator<Item = EntityRef> {
		app.world().iter_entities().filter(move |child| {
			child
				.get::<Parent>()
				.map(|parent| parent.get() == entity)
				.unwrap_or(false)
		})
	}

	#[test]
	fn call_get_or_create_for_mesh_correctly() {
		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache {
					assert: Some(assert),
					..default()
				}
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache::default()
			}
		}

		let (mut app, ..) = setup::<_Agent, _Factory>();
		app.world_mut().spawn(_Agent);
		app.update();

		fn assert(cache: &_Cache<Mesh>) {
			assert_eq!(
				vec![(&TypeId::of::<_Agent>(), _Agent::mesh().primitive_topology())],
				cache
					.args
					.iter()
					.map(|(t, m)| (t, m.primitive_topology()))
					.collect::<Vec<_>>()
			);
		}
	}

	#[test]
	fn call_material_assets_correctly() {
		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache::default()
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache {
					assert: Some(assert),
					..default()
				}
			}
		}

		let (mut app, ..) = setup::<_Agent, _Factory>();
		app.world_mut().spawn(_Agent);

		app.update();

		fn assert(cache: &_Cache<SMat>) {
			assert_eq!(
				vec![(&TypeId::of::<_Agent>(), _Agent::material().base_color)],
				cache
					.args
					.iter()
					.map(|(t, m)| (t, m.base_color))
					.collect::<Vec<_>>()
			);
		}
	}

	#[test]
	fn add_spawn_after_instantiation_component() {
		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache::default()
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache::default()
			}
		}

		let (mut app, ..) = setup::<_Agent, _Factory>();
		let agent = app.world_mut().spawn(_Agent).id();

		app.update();
		let with_children = app
			.world()
			.entity(agent)
			.get::<SpawnAfterInstantiation>()
			.unwrap()
			.clone();
		// Can't compare `SpawnAfterInstantiation` directly (Arc<dyn Fn(..)>), so we apply the spawn
		// function to see that the configured child is spawned correctly
		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				let mut entity = commands.entity(agent);
				entity.with_children(|parent| (with_children.spawn)(parent));
			});

		assert_eq!(
			vec![&_Child],
			children(&app, agent)
				.filter_map(|child| child.get::<_Child>())
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn log_errors() {
		struct _Factory;

		impl GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<Mesh>>,
				_: ResMut<_Storage<Mesh>>,
			) -> impl GetOrCreateAsset<TypeId, Mesh> {
				_Cache::default()
			}
		}

		impl GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId> for _Factory {
			fn create_from(
				_: ResMut<_Assets<SMat>>,
				_: ResMut<_Storage<SMat>>,
			) -> impl GetOrCreateAsset<TypeId, SMat> {
				_Cache::default()
			}
		}

		let (mut app, logger) = setup::<_AgentWithInstantiationError, _Factory>();
		app.world_mut().spawn(_AgentWithInstantiationError);

		app.update();

		let log = app
			.world()
			.entity(logger)
			.get::<FakeErrorLogMany>()
			.unwrap();

		assert_eq!(
			vec![Error {
				msg: "AAA".to_owned(),
				lvl: Level::Warning,
			}],
			log.0
		);
	}
}
