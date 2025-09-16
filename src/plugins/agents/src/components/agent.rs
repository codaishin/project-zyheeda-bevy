mod dto;

use crate::{assets::agent::AgentAsset, components::agent::dto::AgentDto};
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	traits::{
		handles_agents::{AgentNotLoaded, AgentType},
		handles_enemies::EnemyType,
	},
};
use macros::{SavableComponent, agent_asset};

#[derive(Component, SavableComponent, Clone, Debug, PartialEq)]
#[savable_component(dto = AgentDto)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub enum Agent<TAsset = AgentAsset>
where
	TAsset: Asset,
{
	Path(AssetPath<'static>),
	Loading(Handle<TAsset>),
	Loaded(Handle<TAsset>),
}

impl From<AgentType> for Agent {
	fn from(agent_type: AgentType) -> Self {
		Self::Path(match agent_type {
			AgentType::Player => AssetPath::from(agent_asset!("player")),
			AgentType::Enemy(EnemyType::VoidSphere) => AssetPath::from(agent_asset!("void_sphere")),
		})
	}
}

impl<'a> From<&'a Agent> for Result<&'a Handle<AgentAsset>, AgentNotLoaded> {
	fn from(value: &'a Agent) -> Self {
		match value {
			Agent::Loaded(handle) => Ok(handle),
			_ => Err(AgentNotLoaded),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::LazyLock;
	use test_case::test_case;

	static HANDLE: LazyLock<Handle<AgentAsset>> = LazyLock::new(Handle::default);

	#[test_case(Agent::Path(AssetPath::from("my/path.agent")), Err(AgentNotLoaded); "none for path")]
	#[test_case(Agent::Loading(HANDLE.clone()), Err(AgentNotLoaded); "none when loading")]
	#[test_case(Agent::Loaded(HANDLE.clone()), Ok(HANDLE.clone()); "some when loaded")]
	fn get_handle(agent: Agent, expected: Result<Handle<AgentAsset>, AgentNotLoaded>) {
		assert_eq!(
			expected,
			Result::<&Handle<AgentAsset>, AgentNotLoaded>::from(&agent).cloned()
		);
	}
}
