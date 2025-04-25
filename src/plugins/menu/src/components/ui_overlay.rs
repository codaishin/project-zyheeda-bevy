use super::{Label, Quickbar, quickbar_panel::QuickbarPanel};
use crate::{
	tools::PanelState,
	traits::{LoadUi, colors::HasPanelColors, insert_ui_content::InsertUiContent},
};
use bevy::prelude::*;
use common::{tools::keys::slot::SlotKey, traits::iteration::IterFinite};

#[derive(Component)]
#[require(Node(full_screen))]
pub struct UIOverlay;

fn full_screen() -> Node {
	Node {
		width: Val::Percent(100.0),
		height: Val::Percent(100.0),
		flex_direction: FlexDirection::ColumnReverse,
		..default()
	}
}

impl LoadUi<AssetServer> for UIOverlay {
	fn load_ui(_: &mut AssetServer) -> Self {
		UIOverlay
	}
}

impl InsertUiContent for UIOverlay {
	fn insert_ui_content<TLocalization>(&self, _: &mut TLocalization, parent: &mut ChildBuilder) {
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
