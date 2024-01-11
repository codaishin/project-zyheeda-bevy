use std::marker::PhantomData;

use crate::{components::SlotKey, plugins::ingame_menu::tools::PanelState};
use bevy::ecs::component::Component;

#[derive(Component, Debug, PartialEq)]
pub struct InventoryPanel(pub PanelState);

#[derive(Component)]
pub struct InventoryScreen;

#[derive(Component)]
pub struct Label<T, TKey> {
	pub key: TKey,
	phantom_data: PhantomData<T>,
}

impl<T, TKey> Label<T, TKey> {
	pub fn new(key: TKey) -> Self {
		Self {
			key,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: SlotKey,
	pub state: PanelState,
}

#[derive(Component)]
pub struct QuickbarPanelBackground(pub SlotKey);

#[derive(Component)]
pub struct Quickbar;

#[derive(Component)]
pub struct UIOverlay;

#[derive(Component)]
pub struct ColorOverride;
