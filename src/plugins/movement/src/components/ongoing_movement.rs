use bevy::prelude::*;
use common::traits::handles_movement::MovementTarget;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "ongoing movement")]
pub(crate) enum OngoingMovement {
	#[default]
	Stopped,
	Target(MovementTarget),
}

impl OngoingMovement {
	pub(crate) fn target(target: impl Into<MovementTarget>) -> Self {
		Self::Target(target.into())
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct IsMoving;
