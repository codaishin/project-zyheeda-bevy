pub mod inventory_colors;

use bevy::render::color::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BaseColors {
	pub background: Color,
	pub text: Color,
}

pub struct PanelColors {
	pub pressed: Color,
	pub hovered: Color,
	pub empty: Color,
	pub filled: Color,
}

pub trait GetBaseColors {
	fn get_base_colors() -> BaseColors;
}

pub trait GetPanelColors {
	fn get_panel_colors() -> PanelColors;
}
