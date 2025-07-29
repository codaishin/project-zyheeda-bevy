use bevy::prelude::*;
use common::traits::handles_interactions::InteractAble;

#[derive(Component, Debug, PartialEq)]
pub struct Blockable(pub(crate) InteractAble);

impl From<InteractAble> for Blockable {
	fn from(interaction: InteractAble) -> Self {
		Self(interaction)
	}
}
