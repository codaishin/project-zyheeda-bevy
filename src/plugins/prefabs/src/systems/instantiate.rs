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
	traits::shared_asset_handle::SharedAssetHandleProcedural,
};
use std::any::TypeId;

type SMaterial = StandardMaterial;
type GetHandleFn<'a, TAsset> =
	&'a mut dyn FnMut(TypeId, &mut dyn FnMut() -> TAsset) -> Handle<TAsset>;

struct GetHandlesFns<'a> {
	mesh: GetHandleFn<'a, Mesh>,
	material: GetHandleFn<'a, SMaterial>,
}

impl<'a> AssetHandleFor<Mesh> for GetHandlesFns<'a> {
	fn handle<TKey: 'static>(&mut self, new: &mut dyn FnMut() -> Mesh) -> Handle<Mesh> {
		let key = TypeId::of::<TKey>();
		(self.mesh)(key, new)
	}
}

impl<'a> AssetHandleFor<SMaterial> for GetHandlesFns<'a> {
	fn handle<TKey: 'static>(&mut self, new: &mut dyn FnMut() -> SMaterial) -> Handle<SMaterial> {
		let key = TypeId::of::<TKey>();
		(self.material)(key, new)
	}
}

pub fn instantiate<
	TAgent: Component + Instantiate,
	TMeshAssets: Resource + SharedAssetHandleProcedural<TMeshCache, TypeId, Mesh>,
	TMaterialAssets: Resource + SharedAssetHandleProcedural<TMaterialCache, TypeId, SMaterial>,
	TMeshCache: Resource,
	TMaterialCache: Resource,
>(
	mut commands: Commands,
	mut meshes: ResMut<TMeshAssets>,
	mut materials: ResMut<TMaterialAssets>,
	mut mesh_cache: ResMut<TMeshCache>,
	mut material_cache: ResMut<TMaterialCache>,
	agents: Query<(Entity, &TAgent), Added<TAgent>>,
) -> Vec<Result<(), Error>> {
	let mesh_cache = mesh_cache.as_mut();
	let material_cache = material_cache.as_mut();

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
					meshes.handle(mesh_cache, type_id, mesh)
				},
				material: &mut |type_id: TypeId, material: &mut dyn FnMut() -> SMaterial| {
					materials.handle(material_cache, type_id, material)
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
		render::color::Color,
		utils::{default, Uuid},
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

		fn material() -> SMaterial {
			SMaterial::from(Color::BLUE)
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
	struct _Assets<TAsset: Asset> {
		phantom_data: PhantomData<TAsset>,
		args: Vec<(_Cache<TAsset>, TypeId, TAsset)>,
		returns: Handle<TAsset>,
	}

	impl<TAssets: Asset> Default for _Assets<TAssets> {
		fn default() -> Self {
			Self {
				phantom_data: PhantomData,
				args: vec![],
				returns: Handle::default(),
			}
		}
	}

	impl<TAsset: Asset> SharedAssetHandleProcedural<_Cache<TAsset>, TypeId, TAsset>
		for _Assets<TAsset>
	{
		fn handle(
			&mut self,
			cache: &mut _Cache<TAsset>,
			key: TypeId,
			new: impl FnOnce() -> TAsset,
		) -> Handle<TAsset> {
			self.args.push((cache.clone(), key, new()));
			self.returns.clone()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Cache<TAsset> {
		phantom_data: PhantomData<TAsset>,
		id: u32,
	}

	impl<TAsset> Clone for _Cache<TAsset> {
		fn clone(&self) -> Self {
			Self {
				phantom_data: PhantomData,
				id: self.id,
			}
		}
	}

	impl<TAsset> Default for _Cache<TAsset> {
		fn default() -> Self {
			Self {
				phantom_data: PhantomData,
				id: default(),
			}
		}
	}

	fn setup<TAgent: Component + Instantiate>() -> (App, Entity) {
		let mut app = App::new();
		let logger = app.world.spawn_empty().id();
		let instantiate_system = instantiate::<
			TAgent,
			_Assets<Mesh>,
			_Assets<SMaterial>,
			_Cache<Mesh>,
			_Cache<SMaterial>,
		>;
		app.init_resource::<_Assets<Mesh>>();
		app.init_resource::<_Assets<SMaterial>>();
		app.init_resource::<_Cache<Mesh>>();
		app.init_resource::<_Cache<SMaterial>>();
		app.add_systems(
			Update,
			instantiate_system.pipe(fake_log_error_lazy_many(logger)),
		);

		(app, logger)
	}

	#[test]
	fn instantiate_mesh() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		app.insert_resource(_Assets::<Mesh> {
			returns: handle.clone(),
			..default()
		});
		app.update();

		let agent = app.world.entity(agent);
		let result = agent.get::<_Result<Mesh>>();

		assert_eq!(Some(handle), result.map(|r| r.0.clone()));
	}

	#[test]
	fn instantiate_material() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});

		app.insert_resource(_Assets::<SMaterial> {
			returns: handle.clone(),
			..default()
		});
		app.update();

		let agent = app.world.entity(agent);
		let result = agent.get::<_Result<SMaterial>>();

		assert_eq!(Some(handle), result.map(|r| r.0.clone()));
	}

	#[test]
	fn call_mesh_assets_correctly() {
		let (mut app, ..) = setup::<_Agent>();
		app.world.spawn(_Agent);

		app.insert_resource(_Cache::<Mesh> {
			id: 42,
			..default()
		});
		app.update();

		let assets = app.world.resource::<_Assets<Mesh>>();

		assert_eq!(
			vec![(
				42,
				&TypeId::of::<_Agent>(),
				_Agent::mesh().primitive_topology()
			)],
			assets
				.args
				.iter()
				.map(|(cache, type_id, mesh)| (cache.id, type_id, mesh.primitive_topology()))
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn call_material_assets_correctly() {
		let (mut app, ..) = setup::<_Agent>();
		app.world.spawn(_Agent);

		app.insert_resource(_Cache::<SMaterial> {
			id: 42,
			..default()
		});
		app.update();

		let assets = app.world.resource::<_Assets<SMaterial>>();

		assert_eq!(
			vec![(42, &TypeId::of::<_Agent>(), _Agent::material().base_color)],
			assets
				.args
				.iter()
				.map(|(cache, type_id, mesh)| (cache.id, type_id, mesh.base_color))
				.collect::<Vec<_>>()
		);
	}

	#[test]
	fn log_errors() {
		let (mut app, logger) = setup::<_AgentWithInstantiationError>();
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
