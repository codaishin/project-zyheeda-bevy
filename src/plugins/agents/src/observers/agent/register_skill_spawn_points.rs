use crate::{assets::agent_config::AgentConfigAsset, components::agent::Agent};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::EntityContextMut,
	handles_skills_control::{SkillSpawnPoints, SpawnPointsDefinition},
};

impl Agent {
	pub(crate) fn register_skill_spawn_points<TSkills>(
		trigger: Trigger<OnAdd, Self>,
		mut skills: StaticSystemParam<TSkills>,
		agents: Query<&Agent>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TSkills: for<'c> EntityContextMut<SkillSpawnPoints, TContext<'c>: SpawnPointsDefinition>,
	{
		let entity = trigger.target();
		let ctx = TSkills::get_entity_context_mut(&mut skills, entity, SkillSpawnPoints);
		let Some(mut ctx) = ctx else {
			return;
		};
		let Ok(Agent { config_handle, .. }) = agents.get(entity) else {
			return;
		};
		let Some(config) = configs.get(config_handle) else {
			return;
		};

		ctx.insert_spawn_point_definition(config.bones.spawners.clone());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::{AgentConfigAsset, Bones};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{handles_map_generation::AgentType, handles_skill_behaviors::SkillSpawner},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _Skills {
		mock: Mock_Skills,
	}

	#[automock]
	impl SpawnPointsDefinition for _Skills {
		fn insert_spawn_point_definition(&mut self, definition: HashMap<String, SkillSpawner>) {
			self.mock.insert_spawn_point_definition(definition);
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut configs_asset = Assets::default();

		for (id, asset) in configs {
			configs_asset.insert(id, asset);
		}
		app.insert_resource(configs_asset);
		app.add_observer(Agent::register_skill_spawn_points::<Query<&mut _Skills>>);

		app
	}

	#[test]
	fn insert_spawners_definition() {
		let config_handle = new_handle();
		let asset = AgentConfigAsset {
			bones: Bones {
				spawners: HashMap::from([
					("a".to_owned(), SkillSpawner::Neutral),
					("b".to_owned(), SkillSpawner::Slot(SlotKey(42))),
				]),
				..default()
			},
			..default()
		};
		let mut app = setup([(&config_handle, asset)]);

		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle,
			},
			_Skills::new().with_mock(|mock| {
				mock.expect_insert_spawn_point_definition()
					.once()
					.with(eq(HashMap::from([
						(String::from("a"), SkillSpawner::Neutral),
						(String::from("b"), SkillSpawner::Slot(SlotKey(42))),
					])))
					.return_const(());
			}),
		));
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let asset = AgentConfigAsset {
			bones: Bones {
				spawners: HashMap::from([
					("a".to_owned(), SkillSpawner::Neutral),
					("b".to_owned(), SkillSpawner::Slot(SlotKey(42))),
				]),
				..default()
			},
			..default()
		};
		let mut app = setup([(&config_handle, asset)]);

		app.world_mut()
			.spawn(_Skills::new().with_mock(|mock| {
				mock.expect_insert_spawn_point_definition()
					.once()
					.return_const(());
			}))
			.insert(Agent {
				agent_type: AgentType::Player,
				config_handle: config_handle.clone(),
			})
			.insert(Agent {
				agent_type: AgentType::Player,
				config_handle,
			});
	}
}
