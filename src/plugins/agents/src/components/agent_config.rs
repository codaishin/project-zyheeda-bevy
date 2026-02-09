use crate::{assets::agent_config::AgentConfigAsset, components::animate_idle::AnimateIdle};
use bevy::prelude::*;
use common::components::persistent_entity::PersistentEntity;

#[derive(Component, Clone, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(PersistentEntity, Transform, Visibility, AnimateIdle)]
pub struct AgentConfig {
	pub(crate) config_handle: Handle<AgentConfigAsset>,
}
