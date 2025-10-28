use crate::assets::agent_config::AgentConfigAsset;
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{GetProperty, TryApplyOn},
		handles_agents::AgentType,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> InsertFrom for T where T: Component + From<(AgentType, Handle<AgentConfigAsset>)> {}

pub(crate) trait InsertFrom:
	Component + From<(AgentType, Handle<AgentConfigAsset>)> + Sized
{
	fn insert_from<TSource>(
		trigger: Trigger<OnInsert, TSource>,
		asset_server: ResMut<AssetServer>,
		commands: ZyheedaCommands,
		sources: Query<&TSource>,
	) where
		Self: AgentHandle<AssetServer>,
		TSource: Component + GetProperty<AgentType>,
	{
		insert_from_internal::<Self, AssetServer, TSource>(trigger, asset_server, commands, sources)
	}
}

pub(crate) trait AgentHandle<TAssets> {
	fn agent_handle(agent_type: AgentType, assets: &mut TAssets) -> Handle<AgentConfigAsset>;
}

fn insert_from_internal<TAgent, TAssetServer, TSource>(
	trigger: Trigger<OnInsert, TSource>,
	mut asset_server: ResMut<TAssetServer>,
	mut commands: ZyheedaCommands,
	sources: Query<&TSource>,
) where
	TAgent: Component + From<(AgentType, Handle<AgentConfigAsset>)> + AgentHandle<TAssetServer>,
	TAssetServer: Resource,
	TSource: Component + GetProperty<AgentType>,
{
	let entity = trigger.target();
	let Ok(source) = sources.get(entity) else {
		return;
	};
	let agent_type = source.get_property();
	let config_handle = TAgent::agent_handle(agent_type, &mut asset_server);

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(TAgent::from((agent_type, config_handle)));
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assets::agent_config::AgentConfigAsset,
		components::{enemy::void_sphere::VoidSphere, player::Player},
	};
	use common::traits::handles_agents::AgentType;
	use std::sync::LazyLock;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Tag {
		agent_type: AgentType,
	}

	impl GetProperty<AgentType> for _Tag {
		fn get_property(&self) -> AgentType {
			self.agent_type
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Agent {
		agent_type: AgentType,
		handle: Handle<AgentConfigAsset>,
	}

	impl From<(AgentType, Handle<AgentConfigAsset>)> for _Agent {
		fn from((agent_type, handle): (AgentType, Handle<AgentConfigAsset>)) -> Self {
			Self { agent_type, handle }
		}
	}

	static HANDLE: LazyLock<Handle<AgentConfigAsset>> = LazyLock::new(new_handle);

	impl AgentHandle<_AssetServer> for _Agent {
		fn agent_handle(_: AgentType, _: &mut _AssetServer) -> Handle<AgentConfigAsset> {
			HANDLE.clone()
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ConcreteAgent;

	#[derive(Resource)]
	struct _AssetServer;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_AssetServer);
		app.add_observer(insert_from_internal::<_Agent, _AssetServer, _Tag>);

		app
	}

	#[test_case(AgentType::from(Player); "player")]
	#[test_case(AgentType::from(VoidSphere); "void sphere")]
	fn insert_agent_of_type(agent_type: AgentType) {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Tag { agent_type });

		assert_eq!(
			Some(&_Agent {
				agent_type,
				handle: HANDLE.clone()
			}),
			entity.get::<_Agent>(),
		);
	}
}
