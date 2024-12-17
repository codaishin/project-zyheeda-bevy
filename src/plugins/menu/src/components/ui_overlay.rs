use super::{quickbar_panel::QuickbarPanel, Label, Quickbar};
use crate::{
	tools::PanelState,
	traits::{
		colors::HasPanelColors,
		ui_components::{GetUIComponents, GetZIndex, GetZIndexGlobal},
		update_children::UpdateChildren,
		LoadUi,
	},
};
use bevy::prelude::*;
use common::traits::iteration::IterFinite;
use skills::slot_key::SlotKey;

#[derive(Component)]
pub struct UIOverlay;

impl LoadUi<AssetServer> for UIOverlay {
	fn load_ui(_: &mut AssetServer) -> Self {
		UIOverlay
	}
}

impl GetZIndex for UIOverlay {}

impl GetZIndexGlobal for UIOverlay {}

impl GetUIComponents for UIOverlay {
	fn ui_components(&self) -> (Node, BackgroundColor) {
		(
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::ColumnReverse,
				..default()
			},
			default(),
		)
	}
}

impl UpdateChildren for UIOverlay {
	fn update_children(&self, parent: &mut ChildBuilder) {
		add_quickbar(parent);
	}
}

fn add_quickbar(parent: &mut ChildBuilder) {
	parent
		.spawn((
			Quickbar,
			Node {
				width: Val::Percent(500.0),
				height: Val::Px(100.0),
				border: UiRect::all(Val::Px(20.)),
				..default()
			},
		))
		.with_children(|quickbar| {
			for slot_key in SlotKey::iterator() {
				add_slot(quickbar, &slot_key);
			}
		});
}

fn add_slot(quickbar: &mut ChildBuilder, key: &SlotKey) {
	quickbar
		.spawn(Node {
			width: Val::Px(65.0),
			height: Val::Px(65.0),
			margin: UiRect::all(Val::Px(2.0)),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|background| {
			background
				.spawn(get_quickbar_panel(key))
				.with_children(|panel| {
					panel.spawn(get_panel_label(key));
				});
		});
}

fn get_panel_label(key: &SlotKey) -> (Label<QuickbarPanel, SlotKey>, Text, TextFont, TextColor) {
	(
		Label::new(*key),
		Text::new("?"),
		TextFont {
			font_size: 20.0,
			..default()
		},
		TextColor(QuickbarPanel::PANEL_COLORS.text),
	)
}

fn get_quickbar_panel(key: &SlotKey) -> (QuickbarPanel, Button, Node) {
	(
		QuickbarPanel {
			key: *key,
			state: PanelState::Empty,
		},
		Button,
		Node {
			width: Val::Percent(100.),
			height: Val::Percent(100.),
			justify_content: JustifyContent::Start,
			align_items: AlignItems::Start,
			..default()
		},
	)
}
