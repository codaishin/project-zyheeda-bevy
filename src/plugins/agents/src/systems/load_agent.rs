use crate::components::agent::Agent;
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, load_asset::LoadAsset},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl<TAsset> Agent<TAsset>
where
	TAsset: Asset + InsertSpecializedAgent,
{
	pub(crate) fn load(
		commands: ZyheedaCommands,
		asset_server: ResMut<AssetServer>,
		agents: Query<(Entity, &mut Self)>,
		agent_assets: Res<Assets<TAsset>>,
	) {
		load_agent_internal(commands, asset_server, agents, agent_assets)
	}
}

fn load_agent_internal<TAssetServer, TAsset>(
	mut commands: ZyheedaCommands,
	mut asset_server: ResMut<TAssetServer>,
	mut agents: Query<(Entity, &mut Agent<TAsset>)>,
	agent_assets: Res<Assets<TAsset>>,
) where
	TAssetServer: Resource + LoadAsset,
	TAsset: Asset + InsertSpecializedAgent,
{
	for (entity, mut agent) in &mut agents {
		transition_to_loading(&mut agent, &mut asset_server);
		transition_to_loaded(&mut agent, &mut commands, &entity, &agent_assets);
	}
}

pub trait InsertSpecializedAgent {
	fn insert_specialized_agent(&self, entity: &mut ZyheedaEntityCommands);
}

fn transition_to_loading<TAssetServer, TAsset>(
	agent: &mut Mut<Agent<TAsset>>,
	server: &mut ResMut<TAssetServer>,
) where
	TAssetServer: Resource + LoadAsset,
	TAsset: Asset,
{
	let Agent::Path(path) = agent.as_ref() else {
		return;
	};

	**agent = Agent::Loading(server.load_asset(path.clone()));
}

fn transition_to_loaded<TAsset>(
	agent: &mut Mut<Agent<TAsset>>,
	commands: &mut ZyheedaCommands,
	entity: &Entity,
	assets: &Assets<TAsset>,
) where
	TAsset: Asset + InsertSpecializedAgent,
{
	let Agent::Loading(handle) = agent.as_ref() else {
		return;
	};

	let Some(asset) = assets.get(handle) else {
		return;
	};

	**agent = Agent::Loaded(handle.clone());
	commands.try_apply_on(entity, |mut e| {
		asset.insert_specialized_agent(&mut e);
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq)]
	struct _SpecializedAgent;

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

	impl InsertSpecializedAgent for _Asset {
		fn insert_specialized_agent(&self, entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_SpecializedAgent);
		}
	}

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
		loaded_agents: [&Handle<_Asset>; N],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut agent_assets = Assets::default();

		for id in loaded_agents {
			agent_assets.insert(id, _Asset);
		}

		app.insert_resource(agent_assets);
		app.insert_resource(asset_server);
		app.add_systems(Update, load_agent_internal::<_AssetServer, _Asset>);

		app
	}

	#[test]
	fn load_agent() {
		let path = AssetPath::from("my/path.agent");
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(path.clone()))
				.return_const(new_handle::<_Asset>());
		});
		let mut app = setup(server, []);
		app.world_mut().spawn(Agent::<_Asset>::Path(path));

		app.update();
	}

	#[test]
	fn set_agent_loading() {
		let path = AssetPath::from("my/path.agent");
		let handle = new_handle();
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<_Asset, AssetPath>()
				.return_const(handle.clone());
		});
		let mut app = setup(server, []);
		let entity = app.world_mut().spawn(Agent::<_Asset>::Path(path)).id();

		app.update();

		assert_eq!(
			Some(&Agent::Loading(handle)),
			app.world().entity(entity).get::<Agent<_Asset>>(),
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
			app.world().entity(entity).get::<Agent<_Asset>>(),
		);
	}

	#[test]
	fn do_not_set_agent_loaded_when_asset_missing() {
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

	#[test]
	fn insert_specialized_agent() {
		let handle = new_handle();
		let server = _AssetServer::new();
		let mut app = setup(server, [&handle]);
		let entity = app.world_mut().spawn(Agent::Loading(handle.clone())).id();

		app.update();

		assert_eq!(
			Some(&_SpecializedAgent),
			app.world().entity(entity).get::<_SpecializedAgent>(),
		);
	}
}
