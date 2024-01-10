pub mod inventory_colors;

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

pub trait HasPanelColors {
	const PANEL_COLORS: PanelColors;
}
