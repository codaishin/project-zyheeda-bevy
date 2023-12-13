pub mod inventory_colors;

use bevy::render::color::Color;

pub struct PanelColors {
	pub pressed: Color,
	pub hovered: Color,
	pub empty: Color,
	pub filled: Color,
}

pub trait GetPanelColors {
	fn get_panel_colors() -> PanelColors;
}
