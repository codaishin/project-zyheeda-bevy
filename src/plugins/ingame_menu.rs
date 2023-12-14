mod components;
mod systems;
mod tools;
mod traits;

use self::{
	components::{InventoryPanel, InventoryScreen},
	systems::{
		dad::{drag::drag, drop::drop},
		despawn::despawn,
		set_state::set_state,
		spawn::spawn,
		toggle_state::toggle_state,
		update_panels::{colors::panel_colors, states::panel_states},
	},
	tools::MenuState,
};
use crate::{
	components::{Inventory, InventoryKey, Player, SlotKey, Slots, Swap},
	states::{GameRunning, Off, On},
};
use bevy::prelude::*;

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<MenuState>()
			.add_systems(Update, toggle_state::<MenuState, Inventory>)
			.add_systems(
				OnEnter(MenuState::Inventory),
				(spawn::<InventoryScreen>, set_state::<GameRunning, Off>),
			)
			.add_systems(
				OnExit(MenuState::Inventory),
				(despawn::<InventoryScreen>, set_state::<GameRunning, On>),
			)
			.add_systems(
				Update,
				(
					panel_colors::<InventoryPanel>,
					panel_states::<InventoryPanel, InventoryKey, Inventory>,
					panel_states::<InventoryPanel, SlotKey, Slots>,
					drag::<Player, InventoryKey>,
					drag::<Player, SlotKey>,
					drop::<Player, InventoryKey, InventoryKey, Swap<InventoryKey, InventoryKey>>,
					drop::<Player, SlotKey, SlotKey, Swap<SlotKey, SlotKey>>,
					drop::<Player, SlotKey, InventoryKey, Swap<SlotKey, InventoryKey>>,
					drop::<Player, InventoryKey, SlotKey, Swap<InventoryKey, SlotKey>>,
				)
					.run_if(in_state(MenuState::Inventory)),
			);
	}
}
