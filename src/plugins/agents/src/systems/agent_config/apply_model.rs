use crate::{
	assets::agent_meta::{AgentMeta, AgentModel},
	components::{
		agent::{ApplyAgentAnimations, ApplyAgentModel},
		agent_config::AgentConfig,
	},
};
use bevy::prelude::*;
use common::prelude::*;

impl AgentConfig {
	pub(crate) fn apply_model(
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &Self), With<ApplyAgentModel>>,
		metas: Res<Assets<AgentMeta>>,
	) {
		for (entity, AgentConfig { config_handle }) in agents {
			let Some(meta) = metas.get(config_handle) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				match &meta.model {
					AgentModel::Asset(path) => {
						e.try_insert(Model::scene((path, SceneId(0), UseGltfLookup(true))));
					}
					AgentModel::Procedural(proc) => {
						proc(&mut e);
					}
				};

				e.try_insert(ApplyAgentAnimations);
				e.try_remove::<ApplyAgentModel>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::components::agent::{ApplyAgentAnimations, ApplyAgentModel};

	use super::*;
	use bevy::app::App;
	use testing::{SingleThreadedApp, new_handle};

	fn setup<const N: usize>(metas: [(&Handle<AgentMeta>, AgentMeta); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();

		for (id, asset) in metas {
			_ = assets.insert(id, asset);
		}

		app.insert_resource(assets);
		app.add_systems(Update, AgentConfig::apply_model);

		app
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Proc;

	impl _Proc {
		fn insert(e: &mut ZyheedaEntityCommands) {
			e.try_insert(Self);
		}
	}

	#[test]
	fn insert_procedural() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			model: AgentModel::Procedural(_Proc::insert),
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, ApplyAgentModel))
			.id();

		app.update();

		assert!(app.world().entity(entity).contains::<_Proc>());
	}

	#[test]
	fn insert_asset() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			model: AgentModel::Asset(String::from("my/path")),
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, ApplyAgentModel))
			.id();

		app.update();

		assert_eq!(
			Some(&Model::scene(("my/path", SceneId(0), UseGltfLookup(true)))),
			app.world().entity(entity).get::<Model>()
		);
	}

	#[test]
	fn do_nothing_if_marker_missing() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			model: AgentModel::Procedural(_Proc::insert),
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app.world_mut().spawn(AgentConfig { config_handle }).id();

		app.update();

		assert!(!app.world().entity(entity).contains::<_Proc>());
	}

	#[test]
	fn remove_marker() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			model: AgentModel::Procedural(_Proc::insert),
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, ApplyAgentModel))
			.id();

		app.update();

		assert!(!app.world().entity(entity).contains::<ApplyAgentModel>());
	}

	#[test]
	fn insert_animations_marker() {
		let config_handle = new_handle();
		let meta = AgentMeta {
			model: AgentModel::Procedural(_Proc::insert),
			..default()
		};
		let mut app = setup([(&config_handle, meta)]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, ApplyAgentModel))
			.id();

		app.update();

		assert!(
			app.world()
				.entity(entity)
				.contains::<ApplyAgentAnimations>()
		);
	}
}
