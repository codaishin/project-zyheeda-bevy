use bevy::prelude::*;
use common::{
	tools::{Units, UnitsPerSecond},
	traits::animation::Animation,
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct MovementDefinition {
	pub(crate) radius: Units,
	pub(crate) speed: UnitsPerSecond,
	pub(crate) animation: Option<Animation>,
}
