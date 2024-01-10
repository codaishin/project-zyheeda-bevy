use super::Children;
use crate::{
	components::{Side, SlotKey},
	plugins::ingame_menu::{
		components::{Quickbar, QuickbarPanel, UIOverlay},
		tools::PanelState,
		traits::colors::HasPanelColors,
	},
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
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
					..default()
				},
			))
			.with_children(|quickbar| {
				quickbar
					.spawn(get_background())
					.with_children(|background| {
						background.spawn(get_quickbar_bundle(SlotKey::Hand(Side::Main)));
					});
				quickbar
					.spawn(get_background())
					.with_children(|background| {
						background.spawn(get_quickbar_bundle(SlotKey::Hand(Side::Off)));
					});
			});
	}
}

fn get_quickbar_bundle(key: SlotKey) -> (QuickbarPanel, ButtonBundle) {
	(
		QuickbarPanel {
			key,
			state: PanelState::Empty,
		},
		get_panel_button(),
	)
}

fn get_background() -> NodeBundle {
	let slot_style = Style {
		width: Val::Px(65.0),
		height: Val::Px(65.0),
		margin: UiRect::all(Val::Px(2.0)),
		justify_content: JustifyContent::Center,
		align_items: AlignItems::Center,
		..default()
	};
	NodeBundle {
		style: slot_style.clone(),
		background_color: QuickbarPanel::PANEL_COLORS.empty.into(),
		..default()
	}
}

fn get_panel_button() -> ButtonBundle {
	let slot_style = Style {
		width: Val::Percent(100.),
		height: Val::Percent(100.),
		justify_content: JustifyContent::Center,
		align_items: AlignItems::Center,
		..default()
	};
	ButtonBundle {
		style: slot_style.clone(),
		..default()
	}
}
