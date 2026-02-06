use crate::{
	assets::agent_config::{AgentConfigAsset, AgentModel},
	components::agent_config::AgentConfig,
};
use bevy::prelude::*;
use common::{
	components::asset_model::AssetModel,
	traits::accessors::get::TryApplyOn,
	zyheeda_commands::ZyheedaCommands,
};

impl AgentConfig {
	pub(crate) fn insert_model(
		mut commands: ZyheedaCommands,
		configs: Res<Assets<AgentConfigAsset>>,
		agents: Query<(Entity, &Self), Without<ModelInserted>>,
	) {
		for (entity, agent) in &agents {
			let Some(config) = configs.get(&agent.config_handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				match &config.agent_model {
					AgentModel::Asset(path) => {
						e.try_insert(AssetModel::path(path));
					}
					AgentModel::Procedural(func) => {
						func(&mut e);
					}
				};
				e.try_insert(ModelInserted);
			});
		}
	}
}

#[derive(Component)]
pub(crate) struct ModelInserted;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::{AgentConfigAsset, AgentModel};
	use common::zyheeda_commands::ZyheedaEntityCommands;
	use testing::{SingleThreadedApp, new_handle};

	fn setup<const N: usize>(
		model_data: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in model_data {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, AgentConfig::insert_model);

		app
	}

	#[test]
	fn insert_asset_model() {
		let config_handle = new_handle();
		let config = AgentConfigAsset {
			agent_model: AgentModel::from("my/path"),
			..default()
		};
		let mut app = setup([(&config_handle, config)]);
		let entity = app.world_mut().spawn(AgentConfig { config_handle }).id();

		app.update();

		assert_eq!(
			Some(&AssetModel::from("my/path")),
			app.world().entity(entity).get::<AssetModel>()
		);
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Model;

	impl _Model {
		fn insert(e: &mut ZyheedaEntityCommands) {
			e.try_insert(Self);
		}
	}

	#[test]
	fn insert_procedural_model() {
		let config_handle = new_handle();
		let config = AgentConfigAsset {
			agent_model: AgentModel::Procedural(_Model::insert),
			..default()
		};
		let mut app = setup([(&config_handle, config)]);
		let entity = app.world_mut().spawn(AgentConfig { config_handle }).id();

		app.update();

		assert_eq!(Some(&_Model), app.world().entity(entity).get::<_Model>());
	}

	#[test]
	fn insert_model_only_once() {
		let config_handle = new_handle();
		let config = AgentConfigAsset {
			agent_model: AgentModel::Procedural(_Model::insert),
			..default()
		};
		let mut app = setup([(&config_handle, config)]);
		let entity = app.world_mut().spawn(AgentConfig { config_handle }).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Model>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Model>());
	}

	#[test]
	fn insert_model_if_config_is_available_later_than_agent_insertion() {
		let config_handle = new_handle();
		let config = AgentConfigAsset {
			agent_model: AgentModel::Procedural(_Model::insert),
			..default()
		};
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn(AgentConfig {
				config_handle: config_handle.clone(),
			})
			.id();

		app.update();
		_ = app
			.world_mut()
			.resource_mut::<Assets<AgentConfigAsset>>()
			.insert(&config_handle, config);
		app.update();

		assert_eq!(Some(&_Model), app.world().entity(entity).get::<_Model>());
	}
}
