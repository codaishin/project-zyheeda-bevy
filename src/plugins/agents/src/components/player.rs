use bevy::prelude::*;
use common::traits::{handles_animations::AnimationPriority, handles_map_generation::AgentType};

#[derive(Component, Default, Debug, PartialEq, Clone)]
#[component(immutable)]
#[require(Name = "Player")]
pub struct Player;

impl From<Player> for AgentType {
	fn from(_: Player) -> Self {
		Self::Player
	}
}

struct Idle;

impl From<Idle> for AnimationPriority {
	fn from(_: Idle) -> Self {
		AnimationPriority::Low
	}
}
