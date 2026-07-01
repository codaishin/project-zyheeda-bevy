use bevy::prelude::*;
use common::traits::handles_physics::{MouseHover, MouseHoversOver};
use std::collections::HashMap;

#[derive(Resource, Debug, PartialEq, Default)]
pub(crate) struct WorldCamera {
	pub(crate) mouse_hover: HashMap<MouseHover, MouseHoversOver>,
	pub(crate) ray: Option<Ray3d>,
}
