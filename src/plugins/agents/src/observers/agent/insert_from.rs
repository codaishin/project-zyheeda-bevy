use crate::components::agent::Agent;
use bevy::{asset::AssetPath, prelude::*};
use common::{
	traits::{accessors::get::TryApplyOn, handles_agents::AgentType, load_asset::LoadAsset},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl Agent {
	pub(crate) fn insert_self_from<TSource>(
		trigger: Trigger<OnInsert, TSource>,
		asset_server: ResMut<AssetServer>,
		commands: ZyheedaCommands,
		sources: Query<&TSource>,
	) where
		TSource: Component + InsertConcreteAgent,
		for<'a> &'a TSource: Into<AssetPath<'static>> + Into<AgentType>,
	{
		Self::insert_internal(trigger, asset_server, commands, sources)
	}

	fn insert_internal<TAssetServer, TSource>(
		trigger: Trigger<OnInsert, TSource>,
		mut asset_server: ResMut<TAssetServer>,
		mut commands: ZyheedaCommands,
		sources: Query<&TSource>,
	) where
		TAssetServer: Resource + LoadAsset,
		TSource: Component + InsertConcreteAgent,
		for<'a> &'a TSource: Into<AssetPath<'static>> + Into<AgentType>,
	{
		let entity = trigger.target();
		let Ok(source) = sources.get(entity) else {
			return;
		};
		let agent_type: AgentType = source.into();
		let config_path: AssetPath = source.into();
		let config_handle = asset_server.load_asset(config_path);

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Self {
				agent_type,
				config_handle,
			});
			source.insert_concrete_agent(&mut e);
		});
	}
}

pub(crate) trait InsertConcreteAgent {
	fn insert_concrete_agent(&self, entity: &mut ZyheedaEntityCommands);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assets::agent_config::AgentConfigAsset,
		components::{enemy::void_sphere::VoidSphere, player::Player},
	};
	use bevy::asset::AssetPath;
	use common::traits::handles_agents::AgentType;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Tag {
		path: AssetPath<'static>,
		agent_type: AgentType,
	}

	impl From<&_Tag> for AssetPath<'static> {
		fn from(_Tag { path, .. }: &_Tag) -> Self {
			path.clone()
		}
	}

	impl From<&_Tag> for AgentType {
		fn from(_Tag { agent_type, .. }: &_Tag) -> Self {
			*agent_type
		}
	}

	impl InsertConcreteAgent for _Tag {
		fn insert_concrete_agent(&self, entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_ConcreteAgent);
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ConcreteAgent;

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

	fn setup(asset_server: _AssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(asset_server);
		app.add_observer(Agent::insert_internal::<_AssetServer, _Tag>);

		app
	}

	#[test]
	fn load_with_path() {
		let path = AssetPath::from("my/path.agent");
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(path.clone()))
				.return_const(new_handle::<AgentConfigAsset>());
		});
		let mut app = setup(server);

		app.world_mut().spawn(_Tag {
			path,
			agent_type: AgentType::Player,
		});
	}

	#[test_case(AgentType::from(Player); "player")]
	#[test_case(AgentType::from(VoidSphere); "void sphere")]
	fn insert_agent_of_type(agent_type: AgentType) {
		let handle = new_handle();
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<AgentConfigAsset, AssetPath>()
				.return_const(handle.clone());
		});
		let mut app = setup(server);

		let entity = app.world_mut().spawn(_Tag {
			path: AssetPath::from("my/path.agent"),
			agent_type,
		});

		assert_eq!(
			Some(&Agent {
				agent_type,
				config_handle: handle
			}),
			entity.get::<Agent>()
		);
	}

	#[test]
	fn insert_agent_component() {
		let handle = new_handle();
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<AgentConfigAsset, AssetPath>()
				.return_const(handle.clone());
		});
		let mut app = setup(server);

		let entity = app.world_mut().spawn(_Tag {
			path: AssetPath::from("my/path.agent"),
			agent_type: AgentType::Player,
		});

		assert_eq!(Some(&_ConcreteAgent), entity.get::<_ConcreteAgent>());
	}

	#[test]
	fn insert_agent_component_on_reinsert() {
		let handle = new_handle();
		let server = _AssetServer::new().with_mock(|mock| {
			mock.expect_load_asset::<AgentConfigAsset, AssetPath>()
				.return_const(handle.clone());
		});
		let mut app = setup(server);

		let mut entity = app.world_mut().spawn(_Tag {
			path: AssetPath::from("my/path.agent"),
			agent_type: AgentType::Player,
		});
		entity.remove::<_ConcreteAgent>();
		entity.insert(_Tag {
			path: AssetPath::from("my/path.agent"),
			agent_type: AgentType::Player,
		});

		assert_eq!(Some(&_ConcreteAgent), entity.get::<_ConcreteAgent>());
	}
}
