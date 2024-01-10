use crate::{components::SlotKey, plugins::ingame_menu::tools::PanelState};
use bevy::ecs::component::Component;

#[derive(Component, Debug, PartialEq)]
pub struct InventoryPanel(pub PanelState);

#[derive(Component)]
pub struct InventoryScreen;

#[derive(Component)]
pub struct QuickbarPanel {
	pub key: SlotKey,
	pub state: PanelState,
}

#[derive(Component)]
pub struct Quickbar;

#[derive(Component)]
pub struct UIOverlay;
