use bevy::prelude::Component;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum Blockable {
	Fragile,
	Beam,
}
