pub mod inventory_colors;
pub mod ui_overlay_colors;

use bevy::render::color::Color;

pub struct PanelColors {
	pub pressed: Color,
	pub hovered: Color,
	pub empty: Color,
	pub filled: Color,
	pub text: Color,
}

pub trait HasBackgroundColor {
	const BACKGROUND_COLOR: Option<Color>;
}

pub const DEFAULT_PANEL_COLORS: PanelColors = PanelColors {
	pressed: Color::rgb(0.35, 0.75, 0.35),
	hovered: Color::rgb(0.25, 0.25, 0.25),
	filled: Color::rgb(0.15, 0.15, 0.15),
	empty: Color::rgb(0.35, 0.35, 0.35),
	text: Color::rgb(0.9, 0.9, 0.9),
};

pub trait HasPanelColors {
	const PANEL_COLORS: PanelColors;
}

pub trait HasActiveColor {
	const ACTIVE_COLOR: Color;
}
