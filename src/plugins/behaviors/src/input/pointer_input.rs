use crate::{
	Movement,
	PathOrWasd,
	components::movement::velocity_based::VelocityBased,
	systems::movement::{
		insert_process_component::InputProcessComponent,
		parse_pointer_movement::PointMovementInput,
	},
};
use bevy::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct PointerInput(pub(crate) Vec3);

impl From<Vec3> for PointerInput {
	fn from(translation: Vec3) -> Self {
		Self(translation)
	}
}

impl InputProcessComponent for PointerInput {
	type TComponent = Movement<PathOrWasd<VelocityBased>>;
}

impl PointMovementInput for PointerInput {}
