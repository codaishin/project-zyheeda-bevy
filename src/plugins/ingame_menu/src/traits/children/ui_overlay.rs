use super::Children;
use crate::{
	components::{quickbar_panel::QuickbarPanel, Label, Quickbar, UIOverlay},
	tools::PanelState,
	traits::colors::HasPanelColors,
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	text::TextStyle,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
		AlignItems,
		JustifyContent,
		Style,
		UiRect,
		Val,
	},
	utils::default,
};
use common::components::Side;
use skills::components::SlotKey;

impl Children for UIOverlay {
	fn children(parent: &mut ChildBuilder) {
		add_quickbar(parent);
	}
}

fn add_quickbar(parent: &mut ChildBuilder) {
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
			add_slot(quickbar, &SlotKey::Hand(Side::Off));
			add_slot(quickbar, &SlotKey::Hand(Side::Main));
		});
}

fn add_slot(quickbar: &mut ChildBuilder, key: &SlotKey) {
	quickbar
		.spawn(get_background())
		.with_children(|background| {
			background
				.spawn(get_quickbar_panel_bundle(key))
				.with_children(|panel| {
					panel.spawn(get_panel_label(key));
				});
		});
}

fn get_panel_label(key: &SlotKey) -> (Label<QuickbarPanel, SlotKey>, TextBundle) {
	(
		Label::new(*key),
		TextBundle::from_section(
			"?",
			TextStyle {
				font_size: 20.0,
				color: QuickbarPanel::PANEL_COLORS.text,
				..default()
			},
		),
	)
}

fn get_quickbar_panel_bundle(key: &SlotKey) -> (QuickbarPanel, ButtonBundle) {
	(
		QuickbarPanel {
			key: *key,
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
		justify_content: JustifyContent::Start,
		align_items: AlignItems::Start,
		..default()
	};
	ButtonBundle {
		style: slot_style.clone(),
		..default()
	}
}
