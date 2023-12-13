use super::SpawnAble;
use crate::{
	components::{Inventory, Side, Slot, SlotKey, TargetPanel},
	plugins::ingame_menu::{
		components::{InventoryPanel, InventoryScreen},
		tools::PanelState,
		traits::colors::BaseColors,
	},
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

const EQUIPMENT_SLOTS: [(SlotKey, &str); 2] = [
	(SlotKey::Hand(Side::Left), "Left Hand"),
	(SlotKey::Hand(Side::Right), "Right Hand"),
];

impl SpawnAble for InventoryScreen {
	fn bundle(colors: BaseColors) -> (bevy::prelude::NodeBundle, Self) {
		(
			NodeBundle {
				style: Style {
					width: Val::Vw(100.0),
					height: Val::Vh(100.0),
					align_items: AlignItems::Center,
					justify_content: JustifyContent::Center,
					..default()
				},
				background_color: colors.background.into(),
				..default()
			},
			InventoryScreen,
		)
	}

	fn children(colors: BaseColors, parent: &mut ChildBuilder) {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					align_items: AlignItems::Start,
					..default()
				},
				..default()
			})
			.with_children(add_equipment(colors))
			.with_children(add_inventory(colors));
	}
}

fn add_inventory(colors: BaseColors) -> impl Fn(&mut ChildBuilder) {
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
				add_title(parent, "Inventory", colors);
				add::<Inventory>(parent, None, 5, 5, 0, colors);
			});
	}
}

fn add_equipment(colors: BaseColors) -> impl Fn(&mut ChildBuilder) {
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
				add_title(parent, "Equipment", colors);
				for (index, (_, name)) in EQUIPMENT_SLOTS.iter().enumerate() {
					add::<Slot>(parent, Some(name), 1, 1, index, colors);
				}
			});
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str, colors: BaseColors) {
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
					color: colors.text,
					..default()
				},
			));
		});
}

fn add<T: Sync + Send + 'static>(
	parent: &mut ChildBuilder,
	label: Option<&str>,
	x: u32,
	y: u32,
	start_index: usize,
	colors: BaseColors,
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
							color: colors.text,
							..default()
						},
					));
				}
				for _ in 0..x {
					parent
						.spawn((
							TargetPanel::<T>::new(index),
							InventoryPanel::from(PanelState::Empty),
							get_panel_button(),
						))
						.with_children(|parent| {
							parent.spawn(TextBundle::from_section(
								"<Empty>",
								TextStyle {
									font_size: 15.0,
									color: colors.text,
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
