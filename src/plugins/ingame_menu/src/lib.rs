mod components;
mod systems;
mod tools;
mod traits;

use bevy::prelude::*;
use common::{
	components::Player,
	resources::{key_map::KeyMap, language_server::LanguageServer, Shared},
	states::{GameRunning, Off, On},
	systems::log::log_many,
	tools::Factory,
	traits::{cache::get_or_load_asset::LoadAssetCache, load_asset::Path},
};
use components::{
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	quickbar_panel::QuickbarPanel,
	ui_overlay::UIOverlay,
};
use skills::{
	components::{
		combos::Combos,
		combos_time_out::CombosTimeOut,
		inventory::Inventory,
		queue::Queue,
		slots::Slots,
	},
	items::{inventory_key::InventoryKey, slot_key::SlotKey},
};
use systems::{
	dad::{drag::drag, drop::drop},
	despawn::despawn,
	items::swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
	mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
	set_state::set_state,
	set_state_from_input::set_state_from_input,
	spawn::spawn,
	update_children::update_children,
	update_panels::{
		activity_colors_override::panel_activity_colors_override,
		colors::panel_colors,
		container_states::panel_container_states,
		get_quickbar_icons::get_quickbar_icons,
		set_quickbar_icons::set_quickbar_icons,
		update_label_text::update_label_text,
	},
};
use tools::menu_state::MenuState;
use traits::{children::Children, colors::HasBackgroundColor, get_style::GetStyle};

trait AddUI {
	fn add_ui<TComponent>(&mut self, on_state: MenuState) -> &mut Self
	where
		TComponent: Component + Default + GetStyle + Children + HasBackgroundColor;
}

impl AddUI for App {
	fn add_ui<TComponent>(&mut self, on_state: MenuState) -> &mut Self
	where
		TComponent: Component + Default + GetStyle + Children + HasBackgroundColor,
	{
		let spawn_component = (spawn::<TComponent>, update_children::<TComponent>).chain();

		self.add_systems(OnEnter(on_state), spawn_component)
			.add_systems(OnExit(on_state), despawn::<TComponent>)
			.add_systems(Update, update_children::<TComponent>)
	}
}

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		resources(app);
		state_control_systems(app);
		ui_overlay_systems(app);
		inventory_screen_systems(app);
	}
}

fn resources(app: &mut App) {
	app.init_state::<MenuState>()
		.init_resource::<Shared<Path, Handle<Image>>>();
}

fn state_control_systems(app: &mut App) {
	app.add_systems(Update, set_state_from_input::<MenuState>)
		.add_systems(OnExit(MenuState::None), set_state::<GameRunning, Off>)
		.add_systems(OnEnter(MenuState::None), set_state::<GameRunning, On>);
}

fn ui_overlay_systems(app: &mut App) {
	app.add_ui::<UIOverlay>(MenuState::None)
		.add_systems(
			Update,
			(
				get_quickbar_icons::<Queue, Combos, CombosTimeOut>.pipe(
					set_quickbar_icons::<
						AssetServer,
						Shared<Path, Handle<Image>>,
						Factory<LoadAssetCache>,
					>,
				),
				update_label_text::<KeyMap<SlotKey, KeyCode>, LanguageServer, QuickbarPanel>,
				panel_colors::<QuickbarPanel>,
				panel_activity_colors_override::<KeyMap<SlotKey, KeyCode>, Queue, QuickbarPanel>,
			)
				.run_if(in_state(MenuState::None)),
		)
		.add_systems(
			Update,
			(
				set_ui_mouse_context,
				prime_mouse_context::<KeyMap<SlotKey, KeyCode>, QuickbarPanel>,
			),
		);
}

fn inventory_screen_systems(app: &mut App) {
	app.add_ui::<InventoryScreen>(MenuState::Inventory)
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
		);
}
