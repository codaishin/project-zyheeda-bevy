use crate::plugins::ingame_menu::tools::PanelState;
use bevy::ecs::component::Component;

#[derive(Component, Debug, PartialEq)]
pub struct InventoryPanel(pub PanelState);

#[derive(Component)]
pub struct InventoryScreen;
