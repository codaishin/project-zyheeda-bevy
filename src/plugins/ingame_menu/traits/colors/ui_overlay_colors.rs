use super::{HasBackgroundColor, HasPanelColors, PanelColors, DEFAULT_PANEL_COLORS};
use crate::plugins::ingame_menu::components::{QuickbarPanel, UIOverlay};
use bevy::render::color::Color;

impl HasBackgroundColor for UIOverlay {
	const BACKGROUND_COLOR: Option<Color> = None;
}

impl HasPanelColors for QuickbarPanel {
	const PANEL_COLORS: PanelColors = DEFAULT_PANEL_COLORS;
}
