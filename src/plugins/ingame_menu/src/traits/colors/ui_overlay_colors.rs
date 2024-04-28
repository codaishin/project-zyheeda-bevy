use super::HasBackgroundColor;
use crate::components::UIOverlay;
use bevy::render::color::Color;

impl HasBackgroundColor for UIOverlay {
	const BACKGROUND_COLOR: Option<Color> = None;
}
