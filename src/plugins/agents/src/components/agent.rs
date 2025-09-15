use crate::assets::agent::AgentAsset;
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	traits::{handles_agents::AgentType, handles_enemies::EnemyType},
};
use macros::agent_asset;

#[derive(Component, Debug, PartialEq)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub enum Agent {
	Path(AssetPath<'static>),
	HAndle(Handle<AgentAsset>),
}

impl From<AgentType> for Agent {
	fn from(agent_type: AgentType) -> Self {
		Self::Path(match agent_type {
			AgentType::Player => AssetPath::from(agent_asset!("player")),
			AgentType::Enemy(EnemyType::VoidSphere) => AssetPath::from(agent_asset!("void_sphere")),
		})
	}
}

impl<'a> From<&'a Agent> for Option<&'a Handle<AgentAsset>> {
	fn from(value: &'a Agent) -> Self {
		match value {
			Agent::Path(..) => None,
			Agent::HAndle(handle) => Some(handle),
		}
	}
}
