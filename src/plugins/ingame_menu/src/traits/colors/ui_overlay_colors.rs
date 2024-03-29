use super::{
	HasActiveColor,
	HasBackgroundColor,
	HasPanelColors,
	HasQueuedColor,
	PanelColors,
	DEFAULT_PANEL_COLORS,
};
use crate::components::{QuickbarPanel, UIOverlay};
use bevy::render::color::Color;

impl HasBackgroundColor for UIOverlay {
	const BACKGROUND_COLOR: Option<Color> = None;
}

impl HasPanelColors for QuickbarPanel {
	const PANEL_COLORS: PanelColors = PanelColors {
		pressed: Color::ORANGE_RED,
		hovered: DEFAULT_PANEL_COLORS.filled,
		empty: DEFAULT_PANEL_COLORS.empty,
		filled: DEFAULT_PANEL_COLORS.filled,
		text: DEFAULT_PANEL_COLORS.text,
	};
}

impl HasActiveColor for QuickbarPanel {
	const ACTIVE_COLOR: Color = Color::GREEN;
}

impl HasQueuedColor for QuickbarPanel {
	const QUEUED_COLOR: Color = Color::YELLOW_GREEN;
}
