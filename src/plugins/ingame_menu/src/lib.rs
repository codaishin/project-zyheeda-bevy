mod components;
mod systems;
mod tools;
mod traits;

use self::{
	components::{InventoryPanel, InventoryScreen, QuickbarPanel, UIOverlay},
	systems::{
		dad::{drag::drag, drop::drop},
		despawn::despawn,
		mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
		set_state::set_state,
		spawn::spawn,
		toggle_state::toggle_state,
		update_panels::{
			activity_colors_override::panel_activity_colors_override,
			colors::panel_colors,
			container_states::panel_container_states,
			quickbar::quickbar,
			update_label_text::update_label_text,
		},
	},
	tools::MenuState,
};
use bevy::prelude::*;
use common::{
	components::Player,
	states::{GameRunning, Off, On},
	systems::log::log_many,
};
use skills::components::{queue::Queue, Inventory, InventoryKey, SlotKey, Slots};
use systems::items::swap::{
	equipped_items::swap_equipped_items,
	inventory_items::swap_inventory_items,
};

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<MenuState>()
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
					panel_container_states::<InventoryPanel, InventoryKey, Inventory>,
					panel_container_states::<InventoryPanel, SlotKey, Slots>,
					drag::<Player, InventoryKey>,
					drag::<Player, SlotKey>,
					drop::<Player, InventoryKey, InventoryKey>,
					drop::<Player, SlotKey, SlotKey>,
					drop::<Player, SlotKey, InventoryKey>,
					drop::<Player, InventoryKey, SlotKey>,
				)
					.run_if(in_state(MenuState::Inventory)),
			)
			.add_systems(
				Update,
				(swap_equipped_items.pipe(log_many), swap_inventory_items),
			)
			.add_systems(OnEnter(MenuState::None), spawn::<UIOverlay>)
			.add_systems(OnExit(MenuState::None), despawn::<UIOverlay>)
			.add_systems(
				Update,
				(
					quickbar::<Queue>,
					update_label_text::<QuickbarPanel>,
					panel_colors::<QuickbarPanel>,
					panel_activity_colors_override::<Queue, QuickbarPanel>,
				)
					.run_if(in_state(MenuState::None)),
			)
			.add_systems(
				Update,
				(set_ui_mouse_context, prime_mouse_context::<QuickbarPanel>),
			);
	}
}
