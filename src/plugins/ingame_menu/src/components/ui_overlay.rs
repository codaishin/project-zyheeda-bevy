use super::{quickbar_panel::QuickbarPanel, Label, Quickbar};
use crate::{
	tools::PanelState,
	traits::{
		colors::HasPanelColors,
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		LoadUi,
	},
};
use bevy::{
	asset::AssetServer,
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::Component,
	text::TextStyle,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
		AlignItems,
		FlexDirection,
		JustifyContent,
		Style,
		UiRect,
		Val,
	},
	utils::default,
};
use common::traits::iteration::IterFinite;
use skills::slot_key::SlotKey;

#[derive(Component)]
pub struct UIOverlay;

impl LoadUi<AssetServer> for UIOverlay {
	fn load_ui(_: &mut AssetServer) -> Self {
		UIOverlay
	}
}

impl GetNode for UIOverlay {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				flex_direction: FlexDirection::ColumnReverse,
				..default()
			},
			..default()
		}
	}
}

impl InstantiateContentOn for UIOverlay {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
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
			for slot_key in SlotKey::iterator() {
				add_slot(quickbar, &slot_key);
			}
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
