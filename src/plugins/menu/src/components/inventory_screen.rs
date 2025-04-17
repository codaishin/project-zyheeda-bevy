use super::{KeyedPanel, inventory_panel::InventoryPanel};
use crate::{
	tools::PanelState,
	traits::{LoadUi, colors::HasPanelColors, insert_ui_content::InsertUiContent},
};
use bevy::prelude::*;
use common::{
	tools::{inventory_key::InventoryKey, keys::slot::SlotKey},
	traits::{
		get_ui_text::{English, GetUiText, UIText},
		iteration::{IterFinite, IterInfinite},
	},
};

#[derive(Component)]
#[require(Node(full_screen), BackgroundColor(gray))]
pub struct InventoryScreen;

fn full_screen() -> Node {
	Node {
		width: Val::Vw(100.0),
		height: Val::Vh(100.0),
		align_items: AlignItems::Center,
		justify_content: JustifyContent::Center,
		..default()
	}
}

fn gray() -> BackgroundColor {
	BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5))
}

impl LoadUi<AssetServer> for InventoryScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		InventoryScreen
	}
}

impl InsertUiContent for InventoryScreen {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Start,
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
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				margin: UiRect::all(Val::Px(5.0)),
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
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::End,
				margin: UiRect::all(Val::Px(5.0)),
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
		.spawn(Node {
			flex_direction: FlexDirection::Row,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|parent| {
			parent.spawn((
				Text::new(title),
				TextFont {
					font_size: 40.0,
					..default()
				},
				TextColor(InventoryPanel::PANEL_COLORS.text),
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
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				if let UIText::String(label) = grid_label.clone() {
					parent.spawn((
						Text::new(label),
						TextFont {
							font_size: 20.0,
							..default()
						},
						TextColor(InventoryPanel::PANEL_COLORS.text),
					));
				}
				for _ in 0..element_count_x {
					parent
						.spawn((
							Button,
							KeyedPanel(element_key()),
							InventoryPanel::from(PanelState::Empty),
							Node {
								width: Val::Px(65.0),
								height: Val::Px(65.0),
								margin: UiRect::all(Val::Px(2.0)),
								justify_content: JustifyContent::Center,
								align_items: AlignItems::Center,
								..default()
							},
						))
						.with_children(|parent| {
							parent.spawn((
								Text::new("<Empty>"),
								TextFont {
									font_size: 15.0,
									..default()
								},
								TextColor(InventoryPanel::PANEL_COLORS.text),
							));
						});
				}
			});
	}
}
