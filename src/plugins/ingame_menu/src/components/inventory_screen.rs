use super::{inventory_panel::InventoryPanel, KeyedPanel};
use crate::{
	tools::PanelState,
	traits::{
		children::Children,
		colors::{HasBackgroundColor, HasPanelColors},
		get_style::GetStyle,
	},
};
use bevy::{
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::Component,
	render::color::Color,
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
use common::traits::{
	get_ui_text::{English, GetUiText, UIText},
	iteration::{IterFinite, IterInfinite},
};
use skills::items::{inventory_key::InventoryKey, slot_key::SlotKey};

#[derive(Component, Default)]
pub struct InventoryScreen;

impl GetStyle for InventoryScreen {
	fn style(&self) -> Style {
		Style {
			width: Val::Vw(100.0),
			height: Val::Vh(100.0),
			align_items: AlignItems::Center,
			justify_content: JustifyContent::Center,
			..default()
		}
	}
}

impl HasBackgroundColor for InventoryScreen {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}

impl Children for InventoryScreen {
	fn children(&mut self, parent: &mut ChildBuilder) {
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
		let mut keys = InventoryKey::iterator_infinite();
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
				add_grid(parent, UIText::Unmapped, 5, 5, || keys.next_infinite());
			});
	}
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
				for key in SlotKey::iterator() {
					add_grid(parent, English::ui_text(&key), 1, 1, || key);
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

fn add_grid<TKey: Sync + Send + 'static>(
	parent: &mut ChildBuilder,
	grid_label: UIText,
	element_count_x: u32,
	element_count_y: u32,
	mut element_key: impl FnMut() -> TKey,
) {
	for _ in 0..element_count_y {
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
				if let UIText::String(label) = grid_label.clone() {
					parent.spawn(TextBundle::from_section(
						label,
						TextStyle {
							font_size: 20.0,
							color: InventoryPanel::PANEL_COLORS.text,
							..default()
						},
					));
				}
				for _ in 0..element_count_x {
					parent
						.spawn((
							KeyedPanel(element_key()),
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
				}
			});
	}
}

fn get_panel_button() -> ButtonBundle {
	let style = Style {
		width: Val::Px(65.0),
		height: Val::Px(65.0),
		margin: UiRect::all(Val::Px(2.0)),
		justify_content: JustifyContent::Center,
		align_items: AlignItems::Center,
		..default()
	};
	ButtonBundle { style, ..default() }
}
