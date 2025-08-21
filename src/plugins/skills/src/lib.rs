mod behaviors;
mod components;
mod item;
mod skills;
mod systems;
mod tools;
mod traits;

use crate::{
	components::{
		combos::dto::CombosDto,
		combos_time_out::dto::CombosTimeOutDto,
		loadout::Loadout,
		queue::dto::QueueDto,
		slots::visualization::SlotVisualization,
	},
	systems::enqueue::EnqueueSystem,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	tools::{
		action_key::slot::{PlayerSlot, SlotKey},
		inventory_key::InventoryKey,
	},
	traits::{
		handles_combo_menu::{ConfigurePlayerCombos, HandlesComboMenu},
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effects::HandlesAllEffects,
		handles_enemies::HandlesEnemyConfig,
		handles_load_tracking::{DependenciesProgress, HandlesLoadTracking, LoadTrackingInApp},
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
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot, VisibleSlots},
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
use skills::{RunSkillBehavior, Skill, dto::SkillDto};
use std::{hash::Hash, marker::PhantomData};
use systems::{
	advance_active_skill::advance_active_skill,
	combos::{queue_update::ComboQueueUpdate, update::UpdateCombos},
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
	loadout_descriptor::LoadoutDescriptor,
	quickbar_descriptor::get_quickbar_descriptors_for,
};
use tools::combo_descriptor::ComboDescriptor;

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TEnemies, TMenu>
	SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TEnemies,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TEnemies: ThreadSafe + HandlesEnemyConfig,
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
		_: &TEnemies,
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

	fn track_loading<TSlot, TAgent>(app: &mut App)
	where
		TSlot: Eq + Hash + ThreadSafe + From<SlotKey>,
		TAgent: VisibleSlots,
	{
		let all_loaded = SlotVisualization::<TSlot>::all_slots_loaded_for::<TAgent>;
		let track_loaded =
			TLoading::register_load_tracking::<(TSlot, TAgent), LoadingGame, DependenciesProgress>;

		track_loaded().in_app(app, all_loaded);
	}

	fn loadout(&self, app: &mut App) {
		TSaveGame::register_savable_component::<Inventory>(app);
		TSaveGame::register_savable_component::<Slots>(app);

		Self::track_loading::<HandSlot, TPlayers::TPlayer>(app);
		Self::track_loading::<ForearmSlot, TPlayers::TPlayer>(app);
		Self::track_loading::<EssenceSlot, TPlayers::TPlayer>(app);
		Self::track_loading::<HandSlot, TEnemies::TEnemyBehavior>(app);
		Self::track_loading::<ForearmSlot, TEnemies::TEnemyBehavior>(app);
		Self::track_loading::<EssenceSlot, TEnemies::TEnemyBehavior>(app);

		app.add_observer(Loadout::<TPlayers::TPlayer>::insert)
			.add_observer(Loadout::<TEnemies::TEnemyBehavior>::insert)
			.add_systems(
				Update,
				(
					Swapper::system,
					SlotVisualization::<HandSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<HandSlot>::track_slots_for::<TEnemies::TEnemyBehavior>,
					SlotVisualization::<HandSlot>::visualize_items,
					SlotVisualization::<ForearmSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<ForearmSlot>::track_slots_for::<TEnemies::TEnemyBehavior>,
					SlotVisualization::<ForearmSlot>::visualize_items,
					SlotVisualization::<EssenceSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<EssenceSlot>::track_slots_for::<TEnemies::TEnemyBehavior>,
					SlotVisualization::<EssenceSlot>::visualize_items,
				)
					.chain(),
			);
	}

	fn skill_execution(&self, app: &mut App) {
		TSaveGame::register_savable_component::<CombosTimeOut>(app);
		TSaveGame::register_savable_component::<Combos>(app);
		TSaveGame::register_savable_component::<Queue>(app);
		TSaveGame::register_savable_component::<SkillExecuter>(app);

		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TInteractions,
			TBehaviors,
			TPlayers,
		>;

		app.add_systems(
			Update,
			(
				TBehaviors::TSkillUsage::enqueue::<Slots, Queue>,
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

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TEnemies, TMenu> Plugin
	for SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TEnemies,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TEnemies: ThreadSafe + HandlesEnemyConfig,
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
