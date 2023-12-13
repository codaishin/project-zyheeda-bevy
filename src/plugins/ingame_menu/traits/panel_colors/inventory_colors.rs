use super::{GetPanelColors, PanelColors};
use crate::plugins::ingame_menu::tools::InventoryColors;
use bevy::render::color::Color;

impl GetPanelColors for InventoryColors {
	fn get_panel_colors() -> super::PanelColors {
		PanelColors {
			pressed: Color::rgb(0.35, 0.75, 0.35),
			hovered: Color::rgb(0.25, 0.25, 0.25),
			filled: Color::rgb(0.15, 0.15, 0.15),
			empty: Color::rgb(0.35, 0.35, 0.35),
		}
	}
}
