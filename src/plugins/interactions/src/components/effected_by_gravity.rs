use bevy::prelude::{Component, Entity};
use common::tools::UnitsPerSecond;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct EffectedByGravity {
	pub(crate) pulls: Vec<Pull>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pull {
	pub(crate) strength: UnitsPerSecond,
	pub(crate) towards: Entity,
}
