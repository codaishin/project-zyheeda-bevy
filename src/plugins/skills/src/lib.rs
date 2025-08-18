mod behaviors;
mod components;
mod item;
mod skills;
mod systems;
mod tools;
mod traits;

use crate::components::{
	combos::dto::CombosDto,
	combos_time_out::dto::CombosTimeOutDto,
	loadout::Loadout,
	queue::dto::QueueDto,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	tools::{
		action_key::{slot::PlayerSlot, user_input::UserInput},
		inventory_key::InventoryKey,
	},
	traits::{
		handles_combo_menu::{ConfigurePlayerCombos, HandlesComboMenu},
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effect::HandlesAllEffects,
		handles_loadout_menu::{ConfigureInventory, HandlesLoadoutMenu},
		handles_orientation::HandlesOrientation,
		handles_player::{
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
		},
		handles_saving::HandlesSaving,
		handles_settings::HandlesSettings,
		handles_skill_behaviors::HandlesSkillBehaviors,
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	combo_node::ComboNode,
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::Slots,
	swapper::Swapper,
};
use item::{Item, dto::ItemDto};
use skills::{QueuedSkill, RunSkillBehavior, Skill, dto::SkillDto};
use std::marker::PhantomData;
use systems::{
	advance_active_skill::advance_active_skill,
	combos::{queue_update::ComboQueueUpdate, update::UpdateCombos},
	enqueue::enqueue,
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
	get_inputs::get_inputs,
	loadout_descriptor::LoadoutDescriptor,
	quickbar_descriptor::get_quickbar_descriptors_for,
};
use tools::combo_descriptor::ComboDescriptor;

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TMenu>
	SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TMenu: ThreadSafe + HandlesLoadoutMenu + HandlesComboMenu,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TSaveGame,
		_: &TInteractions,
		_: &TLoading,
		_: &TSettings,
		_: &TBehaviors,
		_: &TPlayers,
		_: &TMenu,
	) -> Self {
		Self(PhantomData)
	}

	fn skill_load(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<Skill, SkillDto, LoadingGame>(app);
	}

	fn item_load(&self, app: &mut App) {
		TLoading::register_custom_assets::<Item, ItemDto>(app);
	}

	fn loadout(&self, app: &mut App) {
		TSaveGame::register_savable_component::<Inventory>(app);
		TSaveGame::register_savable_component::<Slots>(app);

		app.register_required_components::<TPlayers::TPlayer, Loadout>()
			.add_prefab_observer::<Loadout, ()>()
			.add_systems(Update, Swapper::system);
	}

	fn skill_execution(&self, app: &mut App) {
		TSaveGame::register_savable_component::<CombosTimeOut>(app);
		TSaveGame::register_savable_component::<Combos>(app);
		TSaveGame::register_savable_component::<Queue>(app);
		TSaveGame::register_savable_component::<SkillExecuter>(app);

		let get_inputs = get_inputs::<TSettings::TKeyMap<PlayerSlot>, ButtonInput<UserInput>>;
		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TInteractions,
			TBehaviors,
			TPlayers,
		>;

		app.add_systems(
			Update,
			(
				get_inputs.pipe(enqueue::<Slots, Queue, QueuedSkill>),
				Combos::update::<Queue>,
				flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
				advance_active_skill::<Queue, TPlayers, TBehaviors, SkillExecuter, Virtual>
					.pipe(OnError::log),
				execute_skill,
				flush::<Queue>,
			)
				.chain()
				.before(TBehaviors::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}

	fn config_menus(&self, app: &mut App) {
		TMenu::loadout_with_swapper::<Swapper>().configure(
			app,
			Inventory::describe_loadout_for::<TPlayers::TPlayer, InventoryKey>,
			Slots::describe_loadout_for::<TPlayers::TPlayer, PlayerSlot>,
		);
		TMenu::configure_quickbar_menu(
			app,
			get_quickbar_descriptors_for::<TPlayers::TPlayer, Slots, Queue, Combos>,
		);
		TMenu::combos_with_skill::<Skill>().configure(
			app,
			ComboDescriptor::describe_combos_for::<TPlayers::TPlayer>,
			Combos::<ComboNode>::update_for::<TPlayers::TPlayer, PlayerSlot>,
		);
	}
}

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TMenu> Plugin
	for SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TMenu: ThreadSafe + HandlesLoadoutMenu + HandlesComboMenu,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.loadout(app);
		self.skill_execution(app);
		self.config_menus(app);
	}
}
