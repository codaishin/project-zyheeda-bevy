use crate::traits::{AssetHandleFor, Instantiate};
use bevy::{
	asset::Handle,
	ecs::{
		component::Component,
		entity::Entity,
		query::Added,
		system::{Commands, Query, ResMut, Resource},
	},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{
	errors::{Error, Level},
	traits::cache::{GetOrCreateAsset, GetOrCreateAssetFactory},
};
use std::any::TypeId;

type SMat = StandardMaterial;
type GetHandleFn<'a, TAsset> =
	&'a mut dyn FnMut(TypeId, &mut dyn FnMut() -> TAsset) -> Handle<TAsset>;

struct GetHandlesFns<'a> {
	mesh: GetHandleFn<'a, Mesh>,
	material: GetHandleFn<'a, SMat>,
}

impl<'a> AssetHandleFor<Mesh> for GetHandlesFns<'a> {
	fn handle<TKey: 'static>(&mut self, new: &mut dyn FnMut() -> Mesh) -> Handle<Mesh> {
		let key = TypeId::of::<TKey>();
		(self.mesh)(key, new)
	}
}

impl<'a> AssetHandleFor<SMat> for GetHandlesFns<'a> {
	fn handle<TKey: 'static>(&mut self, new: &mut dyn FnMut() -> SMat) -> Handle<SMat> {
		let key = TypeId::of::<TKey>();
		(self.material)(key, new)
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
	TAgent: Component + Instantiate,
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
		agent.instantiate(
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
	use crate::traits::AssetHandles;
	use bevy::{
		app::{App, Update},
		asset::{Asset, AssetId, Handle},
		ecs::system::{EntityCommands, IntoSystem},
		math::primitives::Sphere,
		prelude::default,
		render::color::Color,
		utils::Uuid,
	};
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
	};
	use std::marker::PhantomData;

	#[derive(Component)]
	struct _Agent;

	impl _Agent {
		fn mesh() -> Mesh {
			Mesh::from(Sphere { radius: 11. })
		}

		fn material() -> SMat {
			SMat::from(Color::BLUE)
		}
	}

	#[derive(Component)]
	struct _Result<TAsset: Asset>(Handle<TAsset>);

	impl Instantiate for _Agent {
		fn instantiate(
			&self,
			on: &mut EntityCommands,
			mut assets: impl AssetHandles,
		) -> Result<(), Error> {
			on.try_insert((
				_Result(assets.handle::<_Agent>(&mut _Agent::mesh)),
				_Result(assets.handle::<_Agent>(&mut _Agent::material)),
			));
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithInstantiationError;

	impl Instantiate for _AgentWithInstantiationError {
		fn instantiate(&self, _: &mut EntityCommands, _: impl AssetHandles) -> Result<(), Error> {
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

	fn setup<TAgent: Component + Instantiate, TCombine>() -> (App, Entity)
	where
		for<'a> TCombine: GetOrCreateAssetFactory<_Assets<Mesh>, Mesh, _Storage<Mesh>, TypeId>
			+ GetOrCreateAssetFactory<_Assets<SMat>, SMat, _Storage<SMat>, TypeId>
			+ 'static,
	{
		let mut app = App::new();
		let logger = app.world.spawn_empty().id();
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

	#[test]
	fn instantiate_mesh() {
		static HANDLE: Handle<Mesh> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0xe1cdbce7_19f4_4b10_8bf6_80e5ca26f266),
		});

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
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let result = agent.get::<_Result<Mesh>>();

		assert_eq!(Some(HANDLE.clone()), result.map(|r| r.0.clone()));
	}

	#[test]
	fn instantiate_material() {
		static HANDLE: Handle<SMat> = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::from_u128(0xe1cdbce7_19f4_4b10_8bf6_80e5ca26f266),
		});

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
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let result = agent.get::<_Result<SMat>>();

		assert_eq!(Some(HANDLE.clone()), result.map(|r| r.0.clone()));
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
		app.world.spawn(_Agent);
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
		app.world.spawn(_Agent);

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
		app.world.spawn(_AgentWithInstantiationError);

		app.update();

		let log = app.world.entity(logger).get::<FakeErrorLogMany>().unwrap();

		assert_eq!(
			vec![Error {
				msg: "AAA".to_owned(),
				lvl: Level::Warning,
			}],
			log.0
		);
	}
}
