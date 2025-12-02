mod behaviors;
mod components;
mod item;
mod observers;
mod skills;
mod system_parameters;
mod systems;
mod traits;

use crate::{
	components::{
		agent_definition::AgentDefinition,
		combos::dto::CombosDto,
		combos_time_out::dto::CombosTimeOutDto,
		loadout::Loadout,
		queue::dto::QueueDto,
		slots::visualization::SlotVisualization,
	},
	skills::SkillId,
	system_parameters::{
		loadout::{LoadoutPrep, LoadoutReader, LoadoutWriter},
		loadout_activity::LoadoutActivity,
	},
	systems::enqueue::EnqueueSystem,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::log::OnError,
	traits::{
		handles_agents::HandlesAgents,
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_load_tracking::HandlesLoadTracking,
		handles_loadout::HandlesLoadout,
		handles_orientation::{FacingSystemParamMut, HandlesOrientation},
		handles_physics::{HandlesAllPhysicalEffects, HandlesRaycast, RaycastSystemParam},
		handles_player::ConfiguresPlayerSkillAnimations,
		handles_saving::HandlesSaving,
		handles_skill_behaviors::HandlesSkillBehaviors,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::Slots,
};
use item::{Item, dto::ItemDto};
use skills::{RunSkillBehavior, Skill, dto::SkillDto};
use std::marker::PhantomData;
use systems::{
	advance_active_skill::advance_active_skill,
	combos::queue_update::ComboQueueUpdate,
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
};

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TPhysics, TLoading, TBehaviors, TAgents>
	SkillsPlugin<(TSaveGame, TPhysics, TLoading, TBehaviors, TAgents)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesAllPhysicalEffects + HandlesRaycast,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TAgents: ThreadSafe + ConfiguresPlayerSkillAnimations + HandlesAgents,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TSaveGame,
		_: &TPhysics,
		_: &TLoading,
		_: &TBehaviors,
		_: &TAgents,
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

		app.add_observer(Slots::set_self_entity)
			.add_observer(AgentDefinition::insert_from_agent::<TAgents::TAgent>)
			.add_systems(
				Update,
				(
					Loadout::<AgentDefinition>::insert,
					SlotVisualization::<HandSlot>::track_slots_for::<AgentDefinition>,
					SlotVisualization::<HandSlot>::visualize_items,
					SlotVisualization::<ForearmSlot>::track_slots_for::<AgentDefinition>,
					SlotVisualization::<ForearmSlot>::visualize_items,
					SlotVisualization::<EssenceSlot>::track_slots_for::<AgentDefinition>,
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
			TPhysics,
			TBehaviors,
			RaycastSystemParam<TPhysics>,
		>;

		app.add_systems(
			Update,
			(
				TBehaviors::TSkillUsage::enqueue::<Slots, Queue>,
				Combos::update::<Queue>,
				flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
				advance_active_skill::<
					Queue,
					TAgents,
					FacingSystemParamMut<TBehaviors>,
					SkillExecuter,
					Virtual,
				>
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

impl<TSaveGame, TPhysics, TLoading, TBehaviors, TAgents> Plugin
	for SkillsPlugin<(TSaveGame, TPhysics, TLoading, TBehaviors, TAgents)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesAllPhysicalEffects + HandlesRaycast,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation + SystemSetDefinition,
	TAgents: ThreadSafe + ConfiguresPlayerSkillAnimations + HandlesAgents,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.loadout(app);
		self.skill_execution(app);
	}
}

impl<TDependencies> HandlesLoadout for SkillsPlugin<TDependencies> {
	type TSkillID = SkillId;
	type TLoadoutPrep<'w, 's> = LoadoutPrep<'w, 's>;
	type TLoadoutRead<'w, 's> = LoadoutReader<'w, 's>;
	type TLoadoutMut<'w, 's> = LoadoutWriter<'w, 's>;
	type TLoadoutActivity<'w, 's> = LoadoutActivity<'w, 's>;
}
