use bevy::prelude::*;

#[derive(Component, PartialEq, Debug, Clone, Copy)]
#[component(immutable)]
pub(crate) enum Physical {
	Contact,
	Projection,
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct Interactive;
