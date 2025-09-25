use crate::{
	components::{enemy::void_sphere::VoidSphere, player::Player},
	observers::agent::insert_from::InsertConcreteAgent,
};
use bevy::{asset::AssetPath, prelude::*};
use common::{
	traits::{handles_agents::AgentType, handles_enemies::EnemyType},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::{SavableComponent, agent_asset};
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub(crate) struct AgentTag(pub(crate) AgentType);

impl From<&AgentTag> for AssetPath<'static> {
	fn from(AgentTag(agent_type): &AgentTag) -> Self {
		match agent_type {
			AgentType::Player => AssetPath::from(agent_asset!("player")),
			AgentType::Enemy(EnemyType::VoidSphere) => AssetPath::from(agent_asset!("void_sphere")),
		}
	}
}

impl From<&AgentTag> for AgentType {
	fn from(AgentTag(agent_type): &AgentTag) -> Self {
		*agent_type
	}
}

impl InsertConcreteAgent for AgentTag {
	fn insert_concrete_agent(&self, entity: &mut ZyheedaEntityCommands) {
		match self.0 {
			AgentType::Player => entity.try_insert(Player),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert(VoidSphere::enemy()),
		};
	}
}
