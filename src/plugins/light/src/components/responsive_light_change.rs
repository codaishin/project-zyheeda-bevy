use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) enum ResponsiveLightChange {
	Increase,
	Decrease,
}
