mod components;
mod systems;
mod tools;
mod traits;

#[cfg(debug_assertions)]
mod debug;

use std::time::Duration;

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
	combo_overview::ComboOverview,
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	quickbar_panel::QuickbarPanel,
	tooltip::{Tooltip, TooltipUI, TooltipUIControl},
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
	added::added,
	combos::{
		get_combos::get_combos,
		load_combo_icon_image::load_combo_icon_image,
		update_combos::update_combos,
	},
	dad::{drag::drag, drop::drop},
	despawn::despawn,
	items::swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
	mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
	set_state::set_state,
	set_state_from_input::set_state_from_input,
	spawn::spawn,
	tooltip::tooltip,
	tooltip_visibility::tooltip_visibility,
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
use traits::{get_node::GetNode, instantiate_content_on::InstantiateContentOn, SkillDescriptor};

trait AddUI {
	fn add_ui<TComponent>(&mut self, on_state: MenuState) -> &mut Self
	where
		TComponent: Component + Default + GetNode + InstantiateContentOn;
}

impl AddUI for App {
	fn add_ui<TComponent>(&mut self, on_state: MenuState) -> &mut Self
	where
		TComponent: Component + Default + GetNode + InstantiateContentOn,
	{
		let spawn_component = (spawn::<TComponent>, update_children::<TComponent>).chain();

		self.add_systems(OnEnter(on_state), spawn_component)
			.add_systems(OnExit(on_state), despawn::<TComponent>)
			.add_systems(Update, update_children::<TComponent>)
	}
}

trait AddTooltip {
	fn add_tooltip<T>(&mut self) -> &mut Self
	where
		T: Sync + Send + 'static,
		Tooltip<T>: InstantiateContentOn + GetNode;
}

impl AddTooltip for App {
	fn add_tooltip<T>(&mut self) -> &mut Self
	where
		T: Sync + Send + 'static,
		Tooltip<T>: InstantiateContentOn + GetNode,
	{
		self.add_systems(Update, tooltip::<T, TooltipUI, TooltipUIControl, Window>)
	}
}

pub struct IngameMenuPlugin;

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		resources(app);
		state_control_systems(app);
		ui_overlay_systems(app);
		combo_overview_systems(app);
		inventory_screen_systems(app);
		tooltip_systems(app);

		#[cfg(debug_assertions)]
		debug::setup_run_time_display(app);
	}
}

fn resources(app: &mut App) {
	app.init_state::<MenuState>()
		.init_resource::<Shared<Path, Handle<Image>>>()
		.insert_resource(TooltipUIControl {
			tooltip_delay: Duration::from_millis(500),
		});
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

fn combo_overview_systems(app: &mut App) {
	let added_combo_overview = added::<ComboOverview>;
	let get_combos = get_combos::<KeyCode, Combos>;
	let load_combo_icon_image = load_combo_icon_image::<
		KeyCode,
		AssetServer,
		Shared<Path, Handle<Image>>,
		Factory<LoadAssetCache>,
	>;
	let update_combo_overview = update_combos::<KeyCode, ComboOverview>;

	app.add_ui::<ComboOverview>(MenuState::ComboOverview)
		.add_tooltip::<SkillDescriptor<KeyCode, Handle<Image>>>()
		.add_systems(
			Update,
			added_combo_overview
				.pipe(get_combos)
				.pipe(load_combo_icon_image)
				.pipe(update_combo_overview)
				.run_if(in_state(MenuState::ComboOverview)),
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

fn tooltip_systems(app: &mut App) {
	app.add_systems(Update, tooltip_visibility::<Real>);
}
