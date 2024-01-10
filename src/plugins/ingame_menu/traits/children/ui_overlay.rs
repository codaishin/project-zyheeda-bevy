use super::Children;
use crate::plugins::ingame_menu::components::{Quickbar, UIOverlay};
use bevy::{
	hierarchy::ChildBuilder,
	render::color::Color,
	ui::{node_bundles::NodeBundle, Style, UiRect, Val},
	utils::default,
};

impl Children for UIOverlay {
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
