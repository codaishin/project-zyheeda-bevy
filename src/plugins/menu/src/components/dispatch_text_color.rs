use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub(crate) struct DispatchTextColor(pub(crate) Color);

impl From<Color> for DispatchTextColor {
	fn from(color: Color) -> Self {
		Self(color)
	}
}
