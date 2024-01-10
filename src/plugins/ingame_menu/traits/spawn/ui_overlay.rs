use super::Spawn;
use crate::plugins::ingame_menu::components::UIOverlay;
use bevy::{
	ui::{FlexDirection, Style, Val},
	utils::default,
};

impl Spawn for UIOverlay {
	fn spawn() -> (Style, Self) {
		(
			Style {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::ColumnReverse,
				..default()
			},
			Self,
		)
	}
}
