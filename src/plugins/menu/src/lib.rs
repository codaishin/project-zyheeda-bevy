pub mod traits;

mod components;
mod events;
mod systems;
mod tools;
mod visualization;

#[cfg(debug_assertions)]
mod debug;

use bevy::{prelude::*, state::state::FreelyMutableState};
use common::{
	resources::{key_map::KeyMap, language_server::LanguageServer, Shared},
	systems::log::log_many,
	traits::{iteration::IterFinite, load_asset::Path, states::PlayState},
};
use components::{
	button_interaction::ButtonInteraction,
	combo_overview::ComboOverview,
	dropdown::Dropdown,
	inventory_panel::InventoryPanel,
	inventory_screen::InventoryScreen,
	key_select::{AppendSkill, KeySelect, ReKeySkill},
	quickbar_panel::QuickbarPanel,
	skill_button::{DropdownItem, Horizontal, SkillButton, Vertical},
	start_game::StartGame,
	start_menu::StartMenu,
	start_menu_button::StartMenuButton,
	tooltip::{Tooltip, TooltipUI, TooltipUIControl},
	ui_overlay::UIOverlay,
	AppendSkillCommand,
};
use events::DropdownEvent;
use player::components::player::Player;
use skills::{
	components::{
		combos::Combos,
		combos_time_out::CombosTimeOut,
		inventory::Inventory,
		queue::Queue,
		slots::Slots,
	},
	inventory_key::InventoryKey,
	skills::Skill,
	slot_key::SlotKey,
};
use std::time::Duration;
use systems::{
	adjust_global_z_index::adjust_global_z_index,
	combos::{
		update_combo_keys::update_combo_keys,
		update_combo_skills::update_combo_skills,
		update_combos_view::update_combos_view,
		update_combos_view_delete_skill::update_combos_view_delete_skill,
		visualize_invalid_skill::visualize_invalid_skill,
	},
	conditions::{added::added, changed::changed, either::either},
	dad::{drag::drag, drop::drop},
	despawn::despawn,
	dropdown::{
		despawn_when_no_children_pressed::dropdown_despawn_when_no_children_pressed,
		detect_focus_change::dropdown_detect_focus_change,
		events::dropdown_events,
		insert_key_select_dropdown::insert_key_select_dropdown,
		insert_skill_select_dropdown::insert_skill_select_dropdown,
		spawn_focused::dropdown_spawn_focused,
		track_child_dropdowns::dropdown_track_child_dropdowns,
	},
	image_color::image_color,
	insert_key_code_text::insert_key_code_text,
	items::swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
	map_pressed_key_select::map_pressed_key_select,
	mouse_context::{prime::prime_mouse_context, set_ui::set_ui_mouse_context},
	on_release_set::OnReleaseSet,
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
use traits::{
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
	reacts_to_menu_hotkeys::ReactsToMenuHotkeys,
	GetLayout,
	LoadUi,
	RootStyle,
	UI,
};
use visualization::unusable::Unusable;

type SlotKeyMap = KeyMap<SlotKey, KeyCode>;

trait AddUI<TState> {
	fn add_ui<TComponent>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + GetNode + InstantiateContentOn;
}

impl<TState> AddUI<TState> for App
where
	TState: States + Copy,
{
	fn add_ui<TComponent>(&mut self, on_state: TState) -> &mut Self
	where
		TComponent: Component + LoadUi<AssetServer> + GetNode + InstantiateContentOn,
	{
		let spawn_component = (
			spawn::<TComponent, AssetServer>,
			update_children::<TComponent>,
		)
			.chain();

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
		self.add_systems(
			Update,
			(
				tooltip::<T, TooltipUI<T>, TooltipUIControl, Window>,
				tooltip_visibility::<Real, T>,
			),
		)
	}
}

trait AddDropdown {
	fn add_dropdown<TItem>(&mut self) -> &mut Self
	where
		TItem: UI + Sync + Send + 'static,
		Dropdown<TItem>: RootStyle + GetLayout;
}

impl AddDropdown for App {
	fn add_dropdown<TItem>(&mut self) -> &mut Self
	where
		TItem: UI + Sync + Send + 'static,
		Dropdown<TItem>: RootStyle + GetLayout,
	{
		self.add_systems(
			Update,
			(
				dropdown_events::<TItem>,
				dropdown_track_child_dropdowns::<TItem>,
				dropdown_detect_focus_change::<TItem>
					.pipe(dropdown_despawn_when_no_children_pressed::<TItem>)
					.pipe(dropdown_spawn_focused::<TItem>),
			),
		)
	}
}

pub struct MenuPlugin<T> {
	pub start_state: T,
	pub new_game_state: T,
	pub play_state: T,
	pub inventory_state: T,
	pub combo_overview_state: T,
}

impl<TState> MenuPlugin<TState>
where
	TState: States + FreelyMutableState + PlayState + IterFinite + ReactsToMenuHotkeys + Copy,
	KeyCode: TryFrom<TState>,
{
	fn resources(&self, app: &mut App) {
		app.init_resource::<Shared<Path, Handle<Image>>>()
			.insert_resource(TooltipUIControl {
				tooltip_delay: Duration::from_millis(500),
			});
	}

	fn events(&self, app: &mut App) {
		app.add_event::<DropdownEvent>();
	}

	fn state_control(&self, app: &mut App) {
		app.add_systems(Update, set_state_from_input::<TState>);
	}

	fn start_menu(&self, app: &mut App) {
		app.add_ui::<StartMenu>(self.start_state)
			.add_systems(Update, panel_colors::<StartMenuButton>)
			.add_systems(Update, StartGame::on_release_set(self.new_game_state));
	}

	fn ui_overlay(&self, app: &mut App) {
		app.add_ui::<UIOverlay>(self.play_state)
			.add_systems(
				Update,
				(
					get_quickbar_icons::<Queue, Combos, CombosTimeOut>.pipe(set_quickbar_icons),
					update_label_text::<SlotKeyMap, LanguageServer, QuickbarPanel>,
					panel_colors::<QuickbarPanel>,
					panel_activity_colors_override::<SlotKeyMap, Queue, QuickbarPanel>,
				)
					.run_if(in_state(self.play_state)),
			)
			.add_systems(
				Update,
				(
					set_ui_mouse_context,
					prime_mouse_context::<SlotKeyMap, QuickbarPanel>,
				),
			);
	}

	fn combo_overview(&self, app: &mut App) {
		app.add_ui::<ComboOverview>(self.combo_overview_state)
			.add_dropdown::<SkillButton<DropdownItem<Vertical>>>()
			.add_dropdown::<SkillButton<DropdownItem<Horizontal>>>()
			.add_dropdown::<KeySelect<ReKeySkill>>()
			.add_dropdown::<KeySelect<AppendSkill>>()
			.add_tooltip::<Skill>()
			.add_systems(
				Update,
				update_combos_view::<Player, Combos, ComboOverview>
					.run_if(either(added::<ComboOverview>).or(changed::<Player, Combos>))
					.run_if(in_state(self.combo_overview_state)),
			)
			.add_systems(
				Update,
				(
					visualize_invalid_skill::<Player, Slots, Unusable>,
					insert_skill_select_dropdown::<Slots, Vertical>,
					insert_skill_select_dropdown::<Slots, Horizontal>,
					insert_key_select_dropdown::<Player, Combos, AppendSkillCommand>,
					update_combos_view_delete_skill::<Player, Combos>,
					update_combo_skills::<Player, Combos, Vertical>,
					update_combo_skills::<Player, Combos, Horizontal>,
					map_pressed_key_select.pipe(update_combo_keys::<Player, Combos>),
				)
					.run_if(in_state(self.combo_overview_state)),
			);
	}

	fn inventory_screen(&self, app: &mut App) {
		app.add_ui::<InventoryScreen>(self.inventory_state)
			.add_systems(
				Update,
				(
					panel_colors::<InventoryPanel>,
					panel_container_states::<InventoryPanel, InventoryKey, Inventory<Skill>>,
					panel_container_states::<InventoryPanel, SlotKey, Slots>,
					drag::<Player, InventoryKey>,
					drag::<Player, SlotKey>,
					drop::<Player, InventoryKey, InventoryKey>,
					drop::<Player, SlotKey, SlotKey>,
					drop::<Player, SlotKey, InventoryKey>,
					drop::<Player, InventoryKey, SlotKey>,
				)
					.run_if(in_state(self.inventory_state)),
			)
			.add_systems(
				Update,
				(swap_equipped_items.pipe(log_many), swap_inventory_items),
			);
	}

	fn general_systems(&self, app: &mut App) {
		app.add_systems(Update, image_color)
			.add_systems(Update, adjust_global_z_index)
			.add_systems(
				Update,
				insert_key_code_text::<SlotKey, SlotKeyMap, LanguageServer>,
			)
			.add_systems(Last, ButtonInteraction::system);
	}
}

impl<TState> Plugin for MenuPlugin<TState>
where
	TState: States + FreelyMutableState + PlayState + IterFinite + ReactsToMenuHotkeys + Copy,
	KeyCode: TryFrom<TState>,
{
	fn build(&self, app: &mut App) {
		self.resources(app);
		self.events(app);
		self.state_control(app);
		self.start_menu(app);
		self.ui_overlay(app);
		self.combo_overview(app);
		self.inventory_screen(app);
		self.general_systems(app);

		#[cfg(debug_assertions)]
		{
			debug::setup_run_time_display::<TState>(app);
			debug::setup_dropdown_test(app);
		}
	}
}