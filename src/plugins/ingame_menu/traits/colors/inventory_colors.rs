use super::{HasBackgroundColor, HasPanelColors, PanelColors};
use crate::plugins::ingame_menu::components::{InventoryPanel, InventoryScreen};
use bevy::render::color::Color;

impl HasPanelColors for InventoryPanel {
	const PANEL_COLORS: PanelColors = PanelColors {
		pressed: Color::rgb(0.35, 0.75, 0.35),
		hovered: Color::rgb(0.25, 0.25, 0.25),
		filled: Color::rgb(0.15, 0.15, 0.15),
		empty: Color::rgb(0.35, 0.35, 0.35),
		text: Color::rgb(0.9, 0.9, 0.9),
	};
}

impl HasBackgroundColor for InventoryScreen {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}
