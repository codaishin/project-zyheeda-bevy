use super::{KeyedPanel, inventory_panel::InventoryPanel};
use crate::{
	tools::PanelState,
	traits::{LoadUi, colors::HasPanelColors, insert_ui_content::InsertUiContent},
};
use bevy::prelude::*;
use common::{
	tools::{inventory_key::InventoryKey, keys::slot::SlotKey},
	traits::{
		handles_localization::{LocalizeToken, localized::Localized},
		iteration::{IterFinite, IterInfinite},
		thread_safe::ThreadSafe,
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
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken,
	{
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Start,
				..default()
			})
			.with_children(add_equipment(localization))
			.with_children(add_inventory(localization));
	}
}

fn add_inventory<TLocalization>(_: &mut TLocalization) -> impl FnMut(&mut ChildBuilder)
where
	TLocalization: LocalizeToken,
{
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
				add_grid(parent, Localized::from(""), 5, 5, || keys.next_infinite());
			});
	}
}

fn add_equipment<TLocalization>(localization: &mut TLocalization) -> impl FnMut(&mut ChildBuilder)
where
	TLocalization: LocalizeToken,
{
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
					add_grid(
						parent,
						localization.localize_token(key).or_token(),
						1,
						1,
						|| key,
					);
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

fn add_grid<TKey>(
	parent: &mut ChildBuilder,
	grid_label: Localized,
	element_count_x: u32,
	element_count_y: u32,
	mut element_key: impl FnMut() -> TKey,
) where
	TKey: ThreadSafe,
{
	let label = &grid_label;

	for _ in 0..element_count_y {
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				parent.spawn((
					Text::new(label.clone()),
					TextFont {
						font_size: 20.0,
						..default()
					},
					TextColor(InventoryPanel::PANEL_COLORS.text),
				));

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
