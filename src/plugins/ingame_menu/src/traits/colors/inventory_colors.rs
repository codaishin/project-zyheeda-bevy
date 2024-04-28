use super::HasBackgroundColor;
use crate::components::InventoryScreen;
use bevy::render::color::Color;

impl HasBackgroundColor for InventoryScreen {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}
