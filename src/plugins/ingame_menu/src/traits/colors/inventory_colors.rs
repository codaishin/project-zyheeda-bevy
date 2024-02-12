use super::{HasBackgroundColor, HasPanelColors, PanelColors, DEFAULT_PANEL_COLORS};
use crate::components::{InventoryPanel, InventoryScreen};
use bevy::render::color::Color;

impl HasPanelColors for InventoryPanel {
	const PANEL_COLORS: PanelColors = DEFAULT_PANEL_COLORS;
}

impl HasBackgroundColor for InventoryScreen {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}
