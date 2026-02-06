use crate::{
	assets::agent_config::AgentConfigAsset,
	components::agent_config::{AgentConfig, RegisterSkillSpawnPoints},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_skill_physics::{RegisterDefinition, SkillSpawnPoints},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl RegisterSkillSpawnPoints {
	pub(crate) fn execute<TLoadout>(
		mut commands: ZyheedaCommands,
		mut loadout: StaticSystemParam<TLoadout>,
		agents: Query<(Entity, &AgentConfig), With<Self>>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TLoadout: for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: RegisterDefinition>,
	{
		for (entity, AgentConfig { config_handle }) in &agents {
			let key = SkillSpawnPoints { entity };

			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout, key) else {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				ctx.register_definition(config.bones.spawners.clone());
				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::{AgentConfigAsset, Bones};
	use common::{
		tools::{action_key::slot::SlotKey, bone_name::BoneName},
		traits::handles_skill_physics::SkillSpawner,
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

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut configs_asset = Assets::default();

		for (id, asset) in configs {
			_ = configs_asset.insert(id, asset);
		}
		app.insert_resource(configs_asset);
		app.add_systems(
			Update,
			RegisterSkillSpawnPoints::execute::<Query<&mut _Skills>>,
		);

		app
	}

	#[test]
	fn insert_spawners_definition() {
		let config_handle = new_handle();
		let asset = AgentConfigAsset {
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
			AgentConfig { config_handle },
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

		app.update();
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let asset = AgentConfigAsset {
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
			.insert(AgentConfig {
				config_handle: config_handle.clone(),
			})
			.insert(AgentConfig { config_handle });

		app.update();
	}
}
