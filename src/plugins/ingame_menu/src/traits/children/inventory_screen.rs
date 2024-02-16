use super::Children;
use crate::{
	components::{InventoryPanel, InventoryScreen, KeyedPanel},
	tools::PanelState,
	traits::colors::HasPanelColors,
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
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
use common::components::Side;
use skills::components::{InventoryKey, SlotKey};
use std::usize;

const EQUIPMENT_SLOTS: [(SlotKey, &str); 2] = [
	(SlotKey::Hand(Side::Off), "Off Hand"),
	(SlotKey::Hand(Side::Main), "Main Hand"),
];

impl Children for InventoryScreen {
	fn children(parent: &mut ChildBuilder) {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					align_items: AlignItems::Start,
					..default()
				},
				..default()
			})
			.with_children(add_equipment())
			.with_children(add_inventory());
	}
}

fn add_inventory() -> impl Fn(&mut ChildBuilder) {
	move |parent| {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Column,
					align_items: AlignItems::Center,
					margin: UiRect::all(Val::Px(5.0)),
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				add_title(parent, "Inventory");
				add(parent, None, 5, 5, 0, InventoryKey);
			});
	}
}

fn slot_key_from_index(index: usize) -> SlotKey {
	let (key, _) = EQUIPMENT_SLOTS[index];
	key
}

fn add_equipment() -> impl Fn(&mut ChildBuilder) {
	move |parent| {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Column,
					align_items: AlignItems::End,
					margin: UiRect::all(Val::Px(5.0)),
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				add_title(parent, "Equipment");
				for (index, (_, name)) in EQUIPMENT_SLOTS.iter().enumerate() {
					add(parent, Some(name), 1, 1, index, slot_key_from_index);
				}
			});
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle::from_section(
				title,
				TextStyle {
					font_size: 40.0,
					color: InventoryPanel::PANEL_COLORS.text,
					..default()
				},
			));
		});
}

fn add<TKey: Sync + Send + 'static>(
	parent: &mut ChildBuilder,
	label: Option<&str>,
	x: u32,
	y: u32,
	start_index: usize,
	parse_key: fn(usize) -> TKey,
) {
	let mut index = start_index;
	for _ in 0..y {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					align_items: AlignItems::Center,
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				if let Some(label) = label {
					parent.spawn(TextBundle::from_section(
						label,
						TextStyle {
							font_size: 20.0,
							color: InventoryPanel::PANEL_COLORS.text,
							..default()
						},
					));
				}
				for _ in 0..x {
					let key = parse_key(index);
					parent
						.spawn((
							KeyedPanel(key),
							InventoryPanel::from(PanelState::Empty),
							get_panel_button(),
						))
						.with_children(|parent| {
							parent.spawn(TextBundle::from_section(
								"<Empty>",
								TextStyle {
									font_size: 15.0,
									color: InventoryPanel::PANEL_COLORS.text,
									..default()
								},
							));
						});
					index += 1;
				}
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
