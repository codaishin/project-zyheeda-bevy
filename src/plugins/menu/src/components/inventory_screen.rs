use super::{KeyedPanel, inventory_panel::InventoryPanel, menu_background::MenuBackground};
use crate::{
	components::label::{UILabel, UILabelText},
	tools::PanelState,
	traits::{LoadUi, colors::HasPanelColors, insert_ui_content::InsertUiContent},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		inventory_key::InventoryKey,
	},
	traits::{
		handles_localization::{Localize, LocalizeToken, Token, localized::Localized},
		iteration::{IterFinite, IterInfinite},
		thread_safe::ThreadSafe,
	},
};

#[derive(Component)]
#[require(MenuBackground)]
pub struct InventoryScreen;

impl LoadUi<AssetServer> for InventoryScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		InventoryScreen
	}
}

impl InsertUiContent for InventoryScreen {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Start,
				..default()
			})
			.with_children(add_equipment(localize))
			.with_children(add_inventory(localize));
	}
}

fn add_inventory<TLocalization>(
	localize: &TLocalization,
) -> impl FnMut(&mut RelatedSpawnerCommands<ChildOf>)
where
	TLocalization: Localize,
{
	move |parent| {
		let mut keys = InventoryKey::iterator_infinite();
		let inventory = localize.localize_token("inventory").or(|_| "Inventory");

		parent
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			})
			.with_children(|parent| {
				add_title(parent, inventory);
				add_grid(parent, None, 5, 5, || keys.next_infinite());
			});
	}
}

fn add_equipment<TLocalization>(
	localize: &TLocalization,
) -> impl FnMut(&mut RelatedSpawnerCommands<ChildOf>)
where
	TLocalization: Localize,
{
	move |parent| {
		let equipment = localize.localize_token("equipment").or(|_| "Equipment");

		parent
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::End,
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			})
			.with_children(|parent| {
				add_title(parent, equipment);
				for key in PlayerSlot::iterator() {
					add_grid(parent, Some(Token::from(key)), 1, 1, || {
						SlotKey::from(key)
					});
				}
			});
	}
}

fn add_title(parent: &mut RelatedSpawnerCommands<ChildOf>, title: Localized) {
	parent
		.spawn(Node {
			flex_direction: FlexDirection::Row,
			align_items: AlignItems::Center,
			..default()
		})
		.with_children(|parent| {
			parent.spawn((
				Text::from(title),
				TextFont {
					font_size: 40.0,
					..default()
				},
				TextColor(InventoryPanel::PANEL_COLORS.filled.text),
			));
		});
}

fn add_grid<TKey>(
	parent: &mut RelatedSpawnerCommands<ChildOf>,
	grid_label: Option<Token>,
	element_count_x: u32,
	element_count_y: u32,
	mut element_key: impl FnMut() -> TKey,
) where
	TKey: ThreadSafe,
{
	for _ in 0..element_count_y {
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				if let Some(token) = &grid_label {
					parent.spawn((
						TextFont {
							font_size: 20.0,
							..default()
						},
						TextColor(InventoryPanel::PANEL_COLORS.filled.text),
						UILabel(token.clone()),
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
								UILabelText,
								TextFont {
									font_size: 15.0,
									..default()
								},
							));
						});
				}
			});
	}
}
