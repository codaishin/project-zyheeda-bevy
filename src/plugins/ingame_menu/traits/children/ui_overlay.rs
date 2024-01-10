use super::Children;
use crate::{
	components::{Side, SlotKey},
	plugins::ingame_menu::{
		components::{Quickbar, QuickbarPanel, UIOverlay},
		tools::PanelState,
	},
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	render::color::Color,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle},
		AlignItems,
		JustifyContent,
		Style,
		UiRect,
		Val,
	},
	utils::default,
};

impl Children for UIOverlay {
	fn children(parent: &mut ChildBuilder) {
		parent
			.spawn((
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
			))
			.with_children(|quickbar| {
				quickbar.spawn((
					QuickbarPanel {
						key: SlotKey::Hand(Side::Main),
						state: PanelState::Empty,
					},
					get_panel_button(),
				));
				quickbar.spawn((
					QuickbarPanel {
						key: SlotKey::Hand(Side::Off),
						state: PanelState::Empty,
					},
					get_panel_button(),
				));
			});
	}
}

fn get_panel_button() -> ButtonBundle {
	let slot_style = Style {
		width: Val::Px(65.0),
		height: Val::Px(65.0),
		margin: UiRect::all(Val::Px(2.0)),
		justify_content: JustifyContent::Center,
		align_items: AlignItems::Center,
		..default()
	};
	ButtonBundle {
		style: slot_style.clone(),
		..default()
	}
}
