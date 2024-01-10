use super::Spawn;
use crate::plugins::ingame_menu::components::InventoryScreen;
use bevy::{
	ui::{AlignItems, JustifyContent, Style, Val},
	utils::default,
};

impl Spawn for InventoryScreen {
	fn spawn() -> (Style, Self) {
		(
			Style {
				width: Val::Vw(100.0),
				height: Val::Vh(100.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				..default()
			},
			InventoryScreen,
		)
	}
}
