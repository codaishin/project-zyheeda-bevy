pub mod components;
pub mod inventory_key;
pub mod item;
pub mod resources;
pub mod skills;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;

use bevy::prelude::*;
use bundles::{ComboBundle, Loadout};
use common::{
	components::{Collection, Swap},
	resources::key_map::KeyMap,
	states::{game_state::GameState, mouse_context::MouseContext},
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
	tools::slot_key::{Side, SlotKey},
	traits::{
		handles_assets_for_children::HandlesAssetsForChildren,
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effect::HandlesAllEffects,
		handles_lifetime::HandlesLifetime,
		handles_orientation::HandlesOrientation,
		handles_player::{
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
		},
		handles_skill_behaviors::HandlesSkillBehaviors,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
	},
};
use components::{
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	skill_spawners::SkillSpawners,
	slots::{ForearmItemSlots, HandItemSlots, Slots, SubMeshEssenceSlots},
};
use inventory_key::InventoryKey;
use item::{dto::ItemDto, item_type::SkillItemType, Item};
use macros::item_asset;
use skills::{dto::SkillDto, QueuedSkill, RunSkillBehavior, Skill};
use std::{marker::PhantomData, time::Duration};
use systems::{
	advance_active_skill::advance_active_skill,
	enqueue::enqueue,
	equip::equip_item,
	execute::ExecuteSkills,
	flush::flush,
	flush_skill_combos::flush_skill_combos,
	get_inputs::get_inputs,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	update_skill_combos::update_skill_combos,
};

pub struct SkillsPlugin<
	TLifeCycles,
	TInteractions,
	TDispatchChildrenAssets,
	TLoading,
	TBehaviors,
	TPlayers,
>(
	PhantomData<(
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
	)>,
);

impl<TLifeCycles, TInteractions, TDispatchChildrenAssets, TLoading, TBehaviors, TPlayers>
	SkillsPlugin<TLifeCycles, TInteractions, TDispatchChildrenAssets, TLoading, TBehaviors, TPlayers>
where
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
{
	pub fn depends_on(
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TDispatchChildrenAssets,
		_: &TLoading,
		_: &TBehaviors,
		_: &TPlayers,
	) -> Self {
		Self(PhantomData)
	}

	fn skill_load(&self, app: &mut App) {
		TLoading::register_custom_folder_assets::<Skill, SkillDto>(app);
	}

	fn item_load(&self, app: &mut App) {
		TLoading::register_custom_assets::<Item, ItemDto>(app);
	}

	fn skill_slot_load(&self, app: &mut App) {
		TDispatchChildrenAssets::register_child_asset::<Slots, HandItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, ForearmItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, SubMeshEssenceSlots>(app);

		app.add_systems(
			PreUpdate,
			SkillSpawners::track_in_self_and_children::<Name>().system(),
		)
		.add_systems(Update, Self::set_player_items)
		.add_systems(
			Update,
			(
				equip_item::<Inventory, InventoryKey, Collection<Swap<InventoryKey, SlotKey>>>
					.pipe(log_many),
				equip_item::<Inventory, InventoryKey, Collection<Swap<SlotKey, InventoryKey>>>
					.pipe(log_many),
			),
		);
	}

	fn skill_execution(&self, app: &mut App) {
		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TLifeCycles,
			TInteractions,
			TBehaviors,
			TPlayers,
		>;

		app.init_resource::<KeyMap<SlotKey, KeyCode>>()
			.add_systems(
				Update,
				(
					get_inputs::<
						KeyMap<SlotKey, KeyCode>,
						ButtonInput<KeyCode>,
						State<MouseContext<KeyCode>>,
					>
						.pipe(enqueue::<Slots, Queue, QueuedSkill>),
					update_skill_combos::<Combos, Queue>,
					flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
					advance_active_skill::<Queue, TPlayers, TBehaviors, SkillExecuter, Virtual>,
					execute_skill.pipe(log_many),
					flush::<Queue>,
				)
					.chain()
					.run_if(in_state(GameState::Play)),
			)
			.add_systems(
				Update,
				(
					trigger_primed_mouse_context,
					advance_just_triggered_mouse_context,
					release_triggered_mouse_context,
					advance_just_released_mouse_context,
				),
			);
	}

	fn set_player_items(
		mut commands: Commands,
		players: Query<Entity, Added<TPlayers::TPlayer>>,
		asset_server: Res<AssetServer>,
	) {
		let Ok(player) = players.get_single() else {
			return;
		};
		let asset_server = asset_server.as_ref();

		commands.try_insert_on(
			player,
			(
				Self::get_inventory(asset_server),
				Self::get_loadout(asset_server),
				Self::get_combos(),
			),
		);
	}

	fn get_loadout(asset_server: &AssetServer) -> Loadout {
		Loadout::new([
			(
				SlotKey::TopHand(Side::Left),
				Some(asset_server.load(item_asset!("pistol"))),
			),
			(
				SlotKey::BottomHand(Side::Left),
				Some(asset_server.load(item_asset!("pistol"))),
			),
			(
				SlotKey::BottomHand(Side::Right),
				Some(asset_server.load(item_asset!("force_essence"))),
			),
			(
				SlotKey::TopHand(Side::Right),
				Some(asset_server.load(item_asset!("force_essence"))),
			),
		])
	}

	fn get_inventory(asset_server: &AssetServer) -> Inventory {
		Inventory::new([
			Some(asset_server.load(item_asset!("pistol"))),
			Some(asset_server.load(item_asset!("pistol"))),
			Some(asset_server.load(item_asset!("pistol"))),
		])
	}

	fn get_combos() -> ComboBundle {
		let timeout = CombosTimeOut::after(Duration::from_secs(2));

		ComboBundle::with_timeout(timeout)
	}
}

impl<TLifeCycles, TInteractions, TDispatchChildrenAssets, TLoading, TBehaviors, TPlayers> Plugin
	for SkillsPlugin<
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TBehaviors,
		TPlayers,
	>
where
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.skill_slot_load(app);
		self.skill_execution(app);
	}
}
