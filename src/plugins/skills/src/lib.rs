mod behaviors;
mod components;
mod item;
mod observers;
mod resources;
mod skills;
mod systems;
mod traits;

use crate::{
	components::{
		combos::dto::CombosDto,
		combos_time_out::dto::CombosTimeOutDto,
		loadout::Loadout,
		queue::dto::QueueDto,
		slots::visualization::SlotVisualization,
	},
	item::SkillItem,
	systems::enqueue::EnqueueSystem,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	tools::action_key::slot::SlotKey,
	traits::{
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effects::HandlesAllEffects,
		handles_enemies::HandlesEnemies,
		handles_load_tracking::{DependenciesProgress, HandlesLoadTracking, LoadTrackingInApp},
		handles_loadout::HandlesLoadout,
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
	combos::queue_update::ComboQueueUpdate,
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
};

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TEnemies>
	SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TEnemies,
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
	TEnemies: ThreadSafe + HandlesEnemies,
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
		TAgent: VisibleSlots + Component,
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
		Self::track_loading::<HandSlot, TEnemies::TEnemy>(app);
		Self::track_loading::<ForearmSlot, TEnemies::TEnemy>(app);
		Self::track_loading::<EssenceSlot, TEnemies::TEnemy>(app);

		app.add_observer(Slots::set_self_entity)
			.add_observer(Loadout::<TPlayers::TPlayer>::insert)
			.add_observer(Loadout::<TEnemies::TEnemy>::insert)
			.add_systems(
				Update,
				(
					Swapper::system,
					SlotVisualization::<HandSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<HandSlot>::track_slots_for::<TEnemies::TEnemy>,
					SlotVisualization::<HandSlot>::visualize_items,
					SlotVisualization::<ForearmSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<ForearmSlot>::track_slots_for::<TEnemies::TEnemy>,
					SlotVisualization::<ForearmSlot>::visualize_items,
					SlotVisualization::<EssenceSlot>::track_slots_for::<TPlayers::TPlayer>,
					SlotVisualization::<EssenceSlot>::track_slots_for::<TEnemies::TEnemy>,
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
}

impl<TSaveGame, TInteractions, TLoading, TSettings, TBehaviors, TPlayers, TEnemies> Plugin
	for SkillsPlugin<(
		TSaveGame,
		TInteractions,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TEnemies,
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
	TEnemies: ThreadSafe + HandlesEnemies,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.loadout(app);
		self.skill_execution(app);
	}
}

impl<TDependencies> HandlesLoadout for SkillsPlugin<TDependencies> {
	type TItemEntry = SkillItem;
	type TSkill = Skill;
	type TSkills = Vec<Skill>;

	type TInventory = Inventory;
	type TSlots = Slots;
	type TCombos = Combos;
}
