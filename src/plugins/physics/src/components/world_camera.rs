use bevy::prelude::*;
use common::traits::handles_physics::MouseHoversOver;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub struct WorldCamera {
	pub(crate) mouse_hover: HashMap<Excluded, MouseHoversOver>,
	// Tracking camera ray in this component to prevent system param overlap when using `RayCaster`
	pub(crate) ray: Option<Ray3d>,
}

pub(crate) type Excluded = Vec<Entity>;
