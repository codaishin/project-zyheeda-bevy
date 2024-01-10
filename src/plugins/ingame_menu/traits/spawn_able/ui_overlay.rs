use super::SpawnAble;
use crate::plugins::ingame_menu::components::{Quickbar, UIOverlay};
use bevy::{
	hierarchy::ChildBuilder,
	render::color::Color,
	ui::{node_bundles::NodeBundle, FlexDirection, Style, UiRect, Val},
	utils::default,
};

impl SpawnAble for UIOverlay {
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

	fn children(parent: &mut ChildBuilder) {
		parent.spawn((
			Quickbar,
			NodeBundle {
				style: Style {
					width: Val::Percent(500.0),
					height: Val::Px(100.0),
					border: UiRect::all(Val::Px(20.)),
					..default()
				},
				background_color: Color::GREEN.into(),
				..default()
			},
		));
	}
}
