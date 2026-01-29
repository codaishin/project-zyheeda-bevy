use crate::{
	assets::agent_config::AgentConfig,
	components::{agent::Agent, register_agent_loadout_bones::RegisterAgentLoadoutBones},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_loadout::register_loadout_bones::{NoBonesRegistered, RegisterLoadoutBones},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl RegisterAgentLoadoutBones {
	pub(crate) fn execute<TLoadout>(
		mut loadout: StaticSystemParam<TLoadout>,
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &Agent), With<Self>>,
		configs: Res<Assets<AgentConfig>>,
	) where
		TLoadout: for<'c> GetContextMut<NoBonesRegistered, TContext<'c>: RegisterLoadoutBones>,
	{
		for (entity, Agent { config_handle, .. }) in &agents {
			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let key = NoBonesRegistered { entity };
			let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout, key) else {
				continue;
			};

			ctx.register_loadout_bones(
				config.bones.forearm_slots.clone(),
				config.bones.hand_slots.clone(),
				config.bones.essence_slots.clone(),
			);
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_config::{AgentConfig, Bones};
	use common::{
		tools::{action_key::slot::SlotKey, bone_name::BoneName},
		traits::handles_map_generation::AgentType,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _LoadoutHandler {
		mock: Mock_LoadoutHandler,
	}

	#[automock]
	impl RegisterLoadoutBones for _LoadoutHandler {
		fn register_loadout_bones(
			&mut self,
			forearms: HashMap<BoneName, SlotKey>,
			hands: HashMap<BoneName, SlotKey>,
			essences: HashMap<BoneName, SlotKey>,
		) {
			self.mock.register_loadout_bones(forearms, hands, essences);
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfig>, AgentConfig); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, asset) in configs {
			_ = config_assets.insert(id, asset);
		}

		app.insert_resource(config_assets);
		app.add_systems(
			Update,
			RegisterAgentLoadoutBones::execute::<Query<Mut<_LoadoutHandler>>>,
		);

		app
	}

	#[test]
	fn register_bones() {
		let config_handle = new_handle();
		let config = AgentConfig {
			bones: Bones {
				spawners: HashMap::from([]),
				forearm_slots: HashMap::from([(BoneName::from("a"), SlotKey(0))]),
				hand_slots: HashMap::from([(BoneName::from("b"), SlotKey(1))]),
				essence_slots: HashMap::from([(BoneName::from("c"), SlotKey(2))]),
			},
			..default()
		};
		let mut app = setup([(&config_handle, config)]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle,
			},
			_LoadoutHandler::new().with_mock(|mock| {
				mock.expect_register_loadout_bones()
					.times(1)
					.with(
						eq(HashMap::from([(BoneName::from("a"), SlotKey(0))])),
						eq(HashMap::from([(BoneName::from("b"), SlotKey(1))])),
						eq(HashMap::from([(BoneName::from("c"), SlotKey(2))])),
					)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let config = AgentConfig::default();
		let mut app = setup([(&config_handle, config)]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle,
			},
			_LoadoutHandler::new().with_mock(|mock| {
				mock.expect_register_loadout_bones()
					.times(1)
					.return_const(());
			}),
		));

		app.update();
		app.update();
	}

	#[test]
	fn register_bones_when_asset_available_later() {
		let config_handle = new_handle();
		let config = AgentConfig {
			bones: Bones {
				spawners: HashMap::from([]),
				forearm_slots: HashMap::from([(BoneName::from("a"), SlotKey(0))]),
				hand_slots: HashMap::from([(BoneName::from("b"), SlotKey(1))]),
				essence_slots: HashMap::from([(BoneName::from("c"), SlotKey(2))]),
			},
			..default()
		};
		let mut app = setup([]);
		app.world_mut().spawn((
			Agent {
				agent_type: AgentType::Player,
				config_handle: config_handle.clone(),
			},
			_LoadoutHandler::new().with_mock(|mock| {
				mock.expect_register_loadout_bones()
					.times(1)
					.with(
						eq(HashMap::from([(BoneName::from("a"), SlotKey(0))])),
						eq(HashMap::from([(BoneName::from("b"), SlotKey(1))])),
						eq(HashMap::from([(BoneName::from("c"), SlotKey(2))])),
					)
					.return_const(());
			}),
		));

		app.update();
		_ = app
			.world_mut()
			.resource_mut::<Assets<AgentConfig>>()
			.insert(&config_handle, config);
		app.update();
	}
}
