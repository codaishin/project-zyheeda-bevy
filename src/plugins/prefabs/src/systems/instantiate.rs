use crate::traits::{AssetHandleFor, Instantiate};
use bevy::{
	asset::{Assets, Handle},
	ecs::{
		component::Component,
		entity::Entity,
		query::Added,
		system::{Commands, Query, ResMut},
	},
	pbr::StandardMaterial,
	render::mesh::Mesh,
};
use common::{
	errors::{Error, Level},
	resources::Shared,
	traits::cache::Cache,
};
use std::any::TypeId;

impl<TMeshFn, TMaterialFn> AssetHandleFor<Mesh> for (TMeshFn, TMaterialFn)
where
	TMeshFn: FnMut(TypeId, &dyn Fn() -> Mesh) -> Handle<Mesh>,
{
	fn handle<TKey: 'static>(&mut self, mesh: &dyn Fn() -> Mesh) -> Handle<Mesh> {
		(self.0)(TypeId::of::<TKey>(), mesh)
	}
}

impl<TMeshFn, TMaterialFn> AssetHandleFor<StandardMaterial> for (TMeshFn, TMaterialFn)
where
	TMaterialFn: FnMut(TypeId, &dyn Fn() -> StandardMaterial) -> Handle<StandardMaterial>,
{
	fn handle<TKey: 'static>(
		&mut self,
		material: &dyn Fn() -> StandardMaterial,
	) -> Handle<StandardMaterial> {
		(self.1)(TypeId::of::<TKey>(), material)
	}
}

pub fn instantiate<TAgent: Component + Instantiate>(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut shared_meshes: ResMut<Shared<TypeId, Handle<Mesh>>>,
	mut shared_materials: ResMut<Shared<TypeId, Handle<StandardMaterial>>>,
	agents: Query<(Entity, &TAgent), Added<TAgent>>,
) -> Vec<Result<(), Error>> {
	let instantiate = |(entity, agent): (Entity, &TAgent)| {
		let Some(mut entity) = commands.get_entity(entity) else {
			return Err(Error {
				msg: format!("Cannot instantiate prefab, because {entity:?} does not exist",),
				lvl: Level::Error,
			});
		};
		agent.instantiate(
			&mut entity,
			(
				|type_id: TypeId, mesh: &dyn Fn() -> Mesh| {
					shared_meshes.cached(type_id, || meshes.add(mesh()))
				},
				|type_id: TypeId, material: &dyn Fn() -> StandardMaterial| {
					shared_materials.cached(type_id, || materials.add(material()))
				},
			),
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
		hierarchy::{BuildChildren, Parent},
		math::primitives::Sphere,
		render::color::Color,
		utils::default,
	};
	use common::{
		errors::Level,
		systems::log::test_tools::{fake_log_error_lazy_many, FakeErrorLogMany},
	};

	#[derive(Component)]
	struct _Agent;

	impl Instantiate for _Agent {
		fn instantiate(
			&self,
			on: &mut EntityCommands,
			mut assets: impl AssetHandles,
		) -> Result<(), Error> {
			on.try_insert((
				assets.handle::<_Agent>(&|| Mesh::from(Sphere { radius: 11. })),
				assets.handle::<_Agent>(&|| StandardMaterial {
					base_color: Color::BLUE,
					..default()
				}),
			));
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithChildren;

	impl Instantiate for _AgentWithChildren {
		fn instantiate(&self, on: &mut EntityCommands, _: impl AssetHandles) -> Result<(), Error> {
			on.with_children(|parent| {
				parent.spawn_empty();
			});
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

	fn setup<TAgent: Component + Instantiate>() -> (App, Entity) {
		let mut app = App::new();
		let logger = app.world.spawn_empty().id();
		app.init_resource::<Assets<Mesh>>();
		app.init_resource::<Assets<StandardMaterial>>();
		app.init_resource::<Shared<TypeId, Handle<Mesh>>>();
		app.init_resource::<Shared<TypeId, Handle<StandardMaterial>>>();
		app.add_systems(
			Update,
			instantiate::<TAgent>.pipe(fake_log_error_lazy_many(logger)),
		);

		(app, logger)
	}

	fn get_original_asset_from_resources<'a, TAsset: Asset>(
		seek: &AssetId<TAsset>,
		app: &'a App,
	) -> Option<&'a TAsset> {
		let assets = app.world.resource::<Assets<TAsset>>();
		let assets: Vec<_> = assets.iter().collect();
		assets
			.iter()
			.find_map(|(id, asset)| if id == seek { Some(asset) } else { None })
			.cloned()
	}

	#[test]
	fn instantiate_mesh() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let mesh = agent.get::<Handle<Mesh>>().unwrap();
		let mesh = get_original_asset_from_resources(&mesh.id(), &app).unwrap();

		assert_eq!(
			Mesh::from(Sphere { radius: 11. }).primitive_topology(),
			mesh.primitive_topology()
		);
	}

	#[test]
	fn instantiate_material() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let mat = agent.get::<Handle<StandardMaterial>>().unwrap();
		let mat = get_original_asset_from_resources(&mat.id(), &app).unwrap();

		assert_eq!(Color::BLUE, mat.base_color);
	}

	#[test]
	fn instantiate_mesh_through_shared_resource() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let mesh = agent.get::<Handle<Mesh>>();
		let shared_mesh = app
			.world
			.resource::<Shared<TypeId, Handle<Mesh>>>()
			.get(&TypeId::of::<_Agent>());

		assert_eq!(shared_mesh, mesh);
	}

	#[test]
	fn instantiate_material_through_shared_resource() {
		let (mut app, ..) = setup::<_Agent>();
		let agent = app.world.spawn(_Agent).id();

		app.update();

		let agent = app.world.entity(agent);
		let mat = agent.get::<Handle<StandardMaterial>>();
		let shared_mat = app
			.world
			.resource::<Shared<TypeId, Handle<StandardMaterial>>>()
			.get(&TypeId::of::<_Agent>());

		assert_eq!(shared_mat, mat);
	}

	#[test]
	fn only_instantiate_when_agent_new() {
		let (mut app, ..) = setup::<_AgentWithChildren>();
		let agent = app.world.spawn(_AgentWithChildren).id();

		app.update();
		app.update();

		let children = app
			.world
			.iter_entities()
			.filter_map(|c| c.get::<Parent>())
			.filter(|p| p.get() == agent);

		assert_eq!(1, children.count());
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
