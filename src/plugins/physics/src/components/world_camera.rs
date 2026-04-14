use bevy::prelude::*;
use common::traits::handles_physics::{MouseHover, MouseHoversOver};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub struct WorldCamera {
	pub(crate) mouse_hover: HashMap<MouseHover, MouseHoversOver>,
	// Tracking camera ray in this component to prevent system param overlap when using `RayCaster`
	pub(crate) ray: Option<Ray3d>,
}
