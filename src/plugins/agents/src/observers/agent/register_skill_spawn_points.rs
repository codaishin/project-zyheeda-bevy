use crate::{assets::agent_config::AgentConfig, components::agent::Agent};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_skill_physics::{RegisterDefinition, SkillSpawnPoints},
};

impl Agent {
	pub(crate) fn register_skill_spawn_points<TSkills>(
		trigger: On<Add, Self>,
		mut skills: StaticSystemParam<TSkills>,
		agents: Query<&Agent>,
		configs: Res<Assets<AgentConfig>>,
	) where
		TSkills: for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: RegisterDefinition>,
	{
		let entity = trigger.entity;
		let ctx = TSkills::get_context_mut(&mut skills, SkillSpawnPoints { entity });
		let Some(mut ctx) = ctx else {
			return;
		};
		let Ok(Agent { config_handle, .. }) = agents.get(entity) else {
			return;
		};
		let Some(config) = configs.get(config_handle) else {
			return;
		};

		ctx.register_definition(config.bones.spawners.clone());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::{AgentConfig, Bones};
	use common::{
		tools::{action_key::slot::SlotKey, bone_name::BoneName},
		traits::{handles_map_generation::AgentType, handles_skill_physics::SkillSpawner},
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
	impl RegisterDefinition for _Skills {
		fn register_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>) {
			self.mock.register_definition(definition);
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfig>, AgentConfig); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut configs_asset = Assets::default();

		for (id, asset) in configs {
			_ = configs_asset.insert(id, asset);
		}
		app.insert_resource(configs_asset);
		app.add_observer(Agent::register_skill_spawn_points::<Query<&mut _Skills>>);

		app
	}

	#[test]
	fn insert_spawners_definition() {
		let config_handle = new_handle();
		let asset = AgentConfig {
			bones: Bones {
				spawners: HashMap::from([
					(BoneName::from("a"), SkillSpawner::Neutral),
					(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
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
				mock.expect_register_definition()
					.once()
					.with(eq(HashMap::from([
						(BoneName::from("a"), SkillSpawner::Neutral),
						(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
					])))
					.return_const(());
			}),
		));
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let asset = AgentConfig {
			bones: Bones {
				spawners: HashMap::from([
					(BoneName::from("a"), SkillSpawner::Neutral),
					(BoneName::from("b"), SkillSpawner::Slot(SlotKey(42))),
				]),
				..default()
			},
			..default()
		};
		let mut app = setup([(&config_handle, asset)]);

		app.world_mut()
			.spawn(_Skills::new().with_mock(|mock| {
				mock.expect_register_definition().once().return_const(());
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
