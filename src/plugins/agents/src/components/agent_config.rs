use crate::{assets::agent_config::AgentConfigAsset, components::animate_idle::AnimateIdle};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Clone, Debug, PartialEq)]
#[component(immutable)]
#[require(
	PersistentEntity,
	Transform,
	Visibility,
	AnimateIdle,
	InsertAgentModel,
	InsertAgentDefaultLoadout,
	RegisterAgentLoadoutBones,
	RegisterSkillSpawnPoints,
	RegisterAgentAnimations
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

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct InsertAgentModel;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct RegisterAgentAnimations;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct RegisterAgentLoadoutBones;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct RegisterSkillSpawnPoints;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct InsertAgentDefaultLoadout;
