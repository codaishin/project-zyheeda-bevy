use crate::traits::{AssetKey, Instantiate};
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
use common::{errors::Error, resources::Shared};

pub fn instantiate<TAgent: Component + Instantiate>(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut shared_meshes: ResMut<Shared<AssetKey, Handle<Mesh>>>,
	mut shared_materials: ResMut<Shared<AssetKey, Handle<StandardMaterial>>>,
	agents: Query<Entity, Added<TAgent>>,
) -> Vec<Result<(), Error>> {
	let instantiate = |agent| {
		TAgent::instantiate(
			&mut commands.entity(agent),
			|key, mesh| shared_meshes.get_handle(key, || meshes.add(mesh.clone())),
			|key, mat| shared_materials.get_handle(key, || materials.add(mat.clone())),
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
		ecs::system::{EntityCommands, IntoSystem},
		hierarchy::{BuildChildren, Parent},
		render::{color::Color, mesh::shape::UVSphere},
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
			on: &mut EntityCommands,
			mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
			mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
		) -> Result<(), Error> {
			on.insert((
				get_mesh_handle(
					AssetKey::Dummy,
					Mesh::from(UVSphere {
						radius: 11.,
						..default()
					}),
				),
				get_material_handle(
					AssetKey::Dummy,
					StandardMaterial {
						base_color: Color::BLUE,
						..default()
					},
				),
			));
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithChildren;

	impl Instantiate for _AgentWithChildren {
		fn instantiate(
			on: &mut EntityCommands,
			_: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
			_: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
		) -> Result<(), Error> {
			on.with_children(|parent| {
				parent.spawn_empty();
			});
			Ok(())
		}
	}

	#[derive(Component)]
	struct _AgentWithInstantiationError;

	impl Instantiate for _AgentWithInstantiationError {
		fn instantiate(
			_: &mut EntityCommands,
			_: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
			_: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
		) -> Result<(), Error> {
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
		app.init_resource::<Shared<AssetKey, Handle<Mesh>>>();
		app.init_resource::<Shared<AssetKey, Handle<StandardMaterial>>>();
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
			Mesh::from(UVSphere {
				radius: 11.,
				..default()
			})
			.primitive_topology(),
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
			.resource::<Shared<AssetKey, Handle<Mesh>>>()
			.get(&AssetKey::Dummy);

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
			.resource::<Shared<AssetKey, Handle<StandardMaterial>>>()
			.get(&AssetKey::Dummy);

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
