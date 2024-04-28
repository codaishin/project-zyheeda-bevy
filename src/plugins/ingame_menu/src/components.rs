pub(crate) mod inventory_panel;
pub(crate) mod quickbar_panel;

use bevy::ecs::component::Component;
use skills::components::SlotKey;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Dad<T>(pub T);

#[derive(Component, Debug, Clone, Copy)]
pub struct KeyedPanel<TKey>(pub TKey);

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
pub struct QuickbarPanelBackground(pub SlotKey);

#[derive(Component)]
pub struct Quickbar;

#[derive(Component)]
pub struct UIOverlay;

#[derive(Component)]
pub struct ColorOverride;
