use bevy::prelude::*;
use common::traits::{
	accessors::get::{GetProperty, Property},
	handles_movement::MovementTarget,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[savable_component(id = "new movement")]
pub(crate) struct NewMovement {
	pub(crate) target: Option<MovementTarget>,
}

impl NewMovement {
	pub(crate) fn stop() -> Self {
		Self { target: None }
	}

	pub(crate) fn to(target: impl Into<MovementTarget>) -> Self {
		Self {
			target: Some(target.into()),
		}
	}
}

impl GetProperty<Option<MovementTarget>> for NewMovement {
	fn get_property(&self) -> <Option<MovementTarget> as Property>::TValue<'_> {
		self.target
	}
}
