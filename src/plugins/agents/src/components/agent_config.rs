use crate::{assets::agent_config::AgentConfigAsset, components::animate_idle::AnimateIdle};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Clone, Debug, PartialEq, Default)]
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
pub struct AgentConfig {
	pub(crate) config_handle: Handle<AgentConfigAsset>,
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
