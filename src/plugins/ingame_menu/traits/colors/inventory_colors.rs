use super::{BaseColors, GetBaseColors, GetPanelColors, PanelColors};
use crate::plugins::ingame_menu::tools::InventoryColors;
use bevy::render::color::Color;

impl GetPanelColors for InventoryColors {
	fn get_panel_colors() -> PanelColors {
		PanelColors {
			pressed: Color::rgb(0.35, 0.75, 0.35),
			hovered: Color::rgb(0.25, 0.25, 0.25),
			filled: Color::rgb(0.15, 0.15, 0.15),
			empty: Color::rgb(0.35, 0.35, 0.35),
		}
	}
}

impl GetBaseColors for InventoryColors {
	fn get_base_colors() -> BaseColors {
		BaseColors {
			background: Color::rgba(0.5, 0.5, 0.5, 0.5),
			text: Color::rgb(0.9, 0.9, 0.9),
		}
	}
}
