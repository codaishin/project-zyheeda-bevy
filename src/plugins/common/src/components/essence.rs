use bevy::prelude::*;
use macros::serde_model;

#[serde_model]
#[derive(Component, Default, Debug, PartialEq, Clone, Copy)]
#[component(immutable)]
pub enum Essence {
	#[default]
	None,
	Force,
}
