use crate::{
	assets::agent_config::AgentConfigAsset,
	components::{
		animate_idle::AnimateIdle,
		insert_agent_default_loadout::InsertAgentDefaultLoadout,
		register_agent_loadout_bones::RegisterAgentLoadoutBones,
	},
};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Clone, Debug, PartialEq)]
#[component(immutable)]
#[require(
	PersistentEntity,
	Transform,
	Visibility,
	AnimateIdle,
	InsertAgentDefaultLoadout,
	RegisterAgentLoadoutBones
)]
pub struct AgentConfig<TAsset = AgentConfigAsset>
where
	TAsset: Asset,
{
	pub(crate) config_handle: Handle<TAsset>,
}

impl Default for AgentConfig {
	fn default() -> Self {
		Self {
			config_handle: Handle::default(),
		}
	}
}
