mod components;
mod systems;
mod tools;
mod traits;

use bevy::prelude::*;
use common::{
	components::Player,
	resources::Shared,
	states::{GameRunning, Off, On},
	systems::log::log_many,
	traits::load_asset::Path,
};
use components::{
	inventory_panel::InventoryPanel,
	quickbar_panel::QuickbarPanel,
	InventoryScreen,
	UIOverlay,
};
use skills::{
	components::{
		combo_linger::ComboLinger,
		combos::Combos,
		inventory::Inventory,
		queue::Queue,
		slots::Slots,
	},
	items::{InventoryKey, SlotKey},
};
use systems::{
	dad::{drag::drag, drop::drop},
	despawn::despawn,
	items::swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
	mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
	set_state::set_state,
	set_state_from_input::set_state_from_input,
	spawn::spawn,
	update_panels::{
		activity_colors_override::panel_activity_colors_override,
		colors::panel_colors,
		container_states::panel_container_states,
		quickbar::quickbar,
		update_label_text::update_label_text,
	},
};
use tools::{menu_state::MenuState, Icon};

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<MenuState>()
			.init_resource::<Shared<Path, Icon>>()
			.add_systems(Update, set_state_from_input::<MenuState>)
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
					quickbar::<Queue, Combos, ComboLinger, AssetServer>,
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
