use crate::{assets::agent_meta::AgentMeta, components::animate_idle::AnimateIdle};
use bevy::prelude::*;
use common::{components::persistent_entity::PersistentEntity, traits::accessors::get::View};

#[derive(Component, Clone, Debug, PartialEq, Default)]
#[component(immutable)]
#[require(PersistentEntity, Transform, Visibility, AnimateIdle)]
pub struct AgentConfig {
	pub(crate) config_handle: Handle<AgentMeta>,
}

impl View<Handle<AgentMeta>> for AgentConfig {
	fn view(&self) -> &'_ Handle<AgentMeta> {
		&self.config_handle
	}
}
