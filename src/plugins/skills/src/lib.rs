mod behaviors;
mod components;
mod item;
mod skills;
mod system_parameters;
mod systems;
mod traits;

use crate::{
	behaviors::SkillBehaviorConfig,
	components::{
		bone_definitions::BoneDefinitions,
		combos::dto::CombosDto,
		combos_time_out::dto::CombosTimeOutDto,
		held_slots::{Current, HeldSlots, Old},
		queue::dto::QueueDto,
		slots::visualization::SlotVisualization,
	},
	skills::SkillId,
	system_parameters::{
		loadout::{LoadoutPrep, LoadoutReader, LoadoutWriter},
		loadout_activity::{LoadoutActivityReader, LoadoutActivityWriter},
	},
	systems::enqueue::EnqueueSystem,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	traits::{
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_load_tracking::HandlesLoadTracking,
		handles_loadout::HandlesLoadout,
		handles_orientation::{FacingSystemParamMut, HandlesOrientation},
		handles_physics::{HandlesAllPhysicalEffects, HandlesRaycast, RaycastSystemParam},
		handles_saving::HandlesSaving,
		handles_skill_physics::{HandlesSkillPhysics, SkillSpawnerMut},
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
use skills::{Skill, dto::SkillDto};
use std::marker::PhantomData;
use systems::{
	advance_active_skill::advance_active_skill,
	combos::queue_update::ComboQueueUpdate,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
};

pub struct SkillsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TPhysics, TLoading, TBehaviors>
	SkillsPlugin<(TSaveGame, TPhysics, TLoading, TBehaviors)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesAllPhysicalEffects + HandlesSkillPhysics + HandlesRaycast,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TBehaviors: ThreadSafe + HandlesOrientation + SystemSetDefinition,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(_: &TSaveGame, _: &TPhysics, _: &TLoading, _: &TBehaviors) -> Self {
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

		app.add_systems(
			Update,
			(
				SlotVisualization::<HandSlot>::track_slots_for::<BoneDefinitions>,
				SlotVisualization::<HandSlot>::visualize_items,
				SlotVisualization::<ForearmSlot>::track_slots_for::<BoneDefinitions>,
				SlotVisualization::<ForearmSlot>::visualize_items,
				SlotVisualization::<EssenceSlot>::track_slots_for::<BoneDefinitions>,
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

		let execute_skill = SkillExecuter::<SkillBehaviorConfig>::execute_system::<
			SkillSpawnerMut<TPhysics>,
			RaycastSystemParam<TPhysics>,
		>;

		app.add_systems(
			Update,
			(
				Queue::enqueue_system::<Slots>,
				Combos::update::<Queue>,
				flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
				advance_active_skill::<
					Queue,
					FacingSystemParamMut<TBehaviors>,
					SkillExecuter,
					Virtual,
				>,
				execute_skill,
				flush::<Queue>,
				HeldSlots::<Old>::update_from::<Current>,
			)
				.chain()
				.before(TBehaviors::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}
}

impl<TSaveGame, TPhysics, TLoading, TBehaviors> Plugin
	for SkillsPlugin<(TSaveGame, TPhysics, TLoading, TBehaviors)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesAllPhysicalEffects + HandlesSkillPhysics + HandlesRaycast,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets + HandlesLoadTracking,
	TBehaviors: ThreadSafe + HandlesOrientation + SystemSetDefinition,
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
	type TLoadout<'w, 's> = LoadoutReader<'w, 's>;
	type TLoadoutMut<'w, 's> = LoadoutWriter<'w, 's>;
	type TLoadoutActivity<'w, 's> = LoadoutActivityReader<'w, 's>;
	type TLoadoutActivityMut<'w, 's> = LoadoutActivityWriter<'w, 's>;
}
