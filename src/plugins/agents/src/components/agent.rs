pub(crate) mod tag;

use crate::{
	assets::agent_config::AgentConfig,
	components::{
		animate_idle::AnimateIdle,
		enemy::void_sphere::VoidSphere,
		insert_agent_default_loadout::InsertAgentDefaultLoadout,
		player::Player,
		register_agent_loadout_bones::RegisterAgentLoadoutBones,
	},
	observers::agent::{insert_concrete_agent::InsertEnemyOrPlayer, insert_from::AgentHandle},
};
use bevy::prelude::*;
use common::{
	components::{collider_relationship::InteractionTarget, persistent_entity::PersistentEntity},
	traits::{handles_enemies::EnemyType, handles_map_generation::AgentType},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::{SavableComponent, agent_asset};

#[derive(Component, SavableComponent, Clone, Debug, PartialEq)]
#[component(immutable)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	AnimateIdle,
	InsertAgentDefaultLoadout,
	RegisterAgentLoadoutBones
)]
pub struct Agent<TAsset = AgentConfig>
where
	TAsset: Asset,
{
	pub(crate) agent_type: AgentType,
	pub(crate) config_handle: Handle<TAsset>,
}

impl From<(AgentType, Handle<AgentConfig>)> for Agent {
	fn from((agent_type, config_handle): (AgentType, Handle<AgentConfig>)) -> Self {
		Self {
			agent_type,
			config_handle,
		}
	}
}

impl AgentHandle<AssetServer> for Agent {
	fn agent_handle(agent_type: AgentType, assets: &mut AssetServer) -> Handle<AgentConfig> {
		let path = match agent_type {
			AgentType::Player => agent_asset!("player"),
			AgentType::Enemy(EnemyType::VoidSphere) => agent_asset!("void_sphere"),
		};

		assets.load(path)
	}
}

impl InsertEnemyOrPlayer for Agent {
	fn insert_enemy_or_player(&self, mut entity: ZyheedaEntityCommands) {
		match self.agent_type {
			AgentType::Player => entity.try_insert(Player),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert(VoidSphere),
		};
	}
}
