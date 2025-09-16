use crate::{assets::agent::AgentAsset, components::agent::Agent};
use bevy::prelude::*;
use common::traits::load_asset::LoadAsset;

impl Agent {
	pub(crate) fn load(
		asset_server: ResMut<AssetServer>,
		agents: Query<&mut Self>,
		agent_assets: Res<Assets<AgentAsset>>,
	) {
		load_agent_internal(asset_server, agents, agent_assets)
	}
}

fn load_agent_internal<TAssetServer>(
	mut asset_server: ResMut<TAssetServer>,
	mut agents: Query<&mut Agent>,
	agent_assets: Res<Assets<AgentAsset>>,
) where
	TAssetServer: Resource + LoadAsset,
{
	for mut agent in &mut agents {
		match agent.as_ref() {
			Agent::Path(path) => {
				*agent = Agent::Loading(asset_server.load_asset(path.clone()));
			}
			Agent::Loading(handle) if agent_assets.contains(handle) => {
				*agent = Agent::Loaded(handle.clone());
			}
			_ => {}
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup<const N: usize>(
		asset_server: _AssetServer,
		loaded_agents: [&Handle<AgentAsset>; N],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut agent_assets = Assets::default();

		for id in loaded_agents {
			agent_assets.insert(id, AgentAsset::default());
		}

		app.insert_resource(agent_assets);
		app.insert_resource(asset_server);
		app.add_systems(Update, load_agent_internal::<_AssetServer>);

		app
	}

	#[test]
	fn load_agent() {
		let path = AssetPath::from("my/path.agent");
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(path.clone()))
				.return_const(new_handle::<AgentAsset>());
		});
		let mut app = setup(server, []);
		app.world_mut().spawn(Agent::Path(path));

		app.update();
	}

	#[test]
	fn set_agent_loading() {
		let path = AssetPath::from("my/path.agent");
		let handle = new_handle();
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<AgentAsset, AssetPath>()
				.return_const(handle.clone());
		});
		let mut app = setup(server, []);
		let entity = app.world_mut().spawn(Agent::Path(path)).id();

		app.update();

		assert_eq!(
			Some(&Agent::Loading(handle)),
			app.world().entity(entity).get::<Agent>(),
		);
	}

	#[test]
	fn set_agent_loaded() {
		let handle = new_handle();
		let server = _AssetServer::new();
		let mut app = setup(server, [&handle]);
		let entity = app.world_mut().spawn(Agent::Loading(handle.clone())).id();

		app.update();

		assert_eq!(
			Some(&Agent::Loaded(handle)),
			app.world().entity(entity).get::<Agent>(),
		);
	}

	#[test]
	fn don_not_set_agent_loaded_when_asset_missing() {
		let handle = new_handle();
		let server = _AssetServer::new();
		let mut app = setup(server, []);
		let entity = app.world_mut().spawn(Agent::Loading(handle.clone())).id();

		app.update();

		assert_eq!(
			Some(&Agent::Loading(handle)),
			app.world().entity(entity).get::<Agent>(),
		);
	}
}
