pub mod components;
pub mod inventory_key;
pub mod item;
pub mod resources;
pub mod skills;
pub mod slot_key;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;

use bevy::prelude::*;
use bundles::{ComboBundle, Loadout};
use common::{
	components::{Collection, Side, Swap},
	resources::key_map::KeyMap,
	states::{game_state::GameState, mouse_context::MouseContext},
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
	traits::{
		animation::HasAnimationsDispatch,
		handles_effect::HandlesAllEffects,
		handles_effect_shading::HandlesEffectShadingForAll,
		handles_lifetime::HandlesLifetime,
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
	slots::Slots,
};
use inventory_key::InventoryKey;
use item::{dto::ItemDto, item_type::SkillItemType, Item};
use loading::traits::{
	register_custom_assets::RegisterCustomAssets,
	register_custom_folder_assets::RegisterCustomFolderAssets,
};
use macros::item_asset;
use player::components::player::Player;
use skills::{dto::SkillDto, QueuedSkill, RunSkillBehavior, Skill};
use slot_key::SlotKey;
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

pub struct SkillsPlugin<TAnimationsPlugin, TLifeCyclePlugin, TInteractionsPlugin, TShadersPlugin>(
	PhantomData<(
		TAnimationsPlugin,
		TLifeCyclePlugin,
		TInteractionsPlugin,
		TShadersPlugin,
	)>,
);

impl<TAnimationsPlugin, TLifeCyclePlugin, TInteractionsPlugin, TShadersPlugin>
	SkillsPlugin<TAnimationsPlugin, TLifeCyclePlugin, TInteractionsPlugin, TShadersPlugin>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TLifeCyclePlugin: Plugin + HandlesLifetime,
	TInteractionsPlugin: Plugin + HandlesAllEffects,
	TShadersPlugin: Plugin + HandlesEffectShadingForAll,
{
	pub fn depends_on(
		_: &TAnimationsPlugin,
		_: &TLifeCyclePlugin,
		_: &TShadersPlugin,
		_: &TInteractionsPlugin,
	) -> Self {
		Self(
			PhantomData::<(
				TAnimationsPlugin,
				TLifeCyclePlugin,
				TInteractionsPlugin,
				TShadersPlugin,
			)>,
		)
	}

	fn skill_load(&self, app: &mut App) {
		app.register_custom_folder_assets::<Skill, SkillDto>();
	}

	fn item_load(&self, app: &mut App) {
		app.register_custom_assets::<Item, ItemDto>();
	}

	fn skill_slot_load(&self, app: &mut App) {
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
			TLifeCyclePlugin,
			TInteractionsPlugin,
			TShadersPlugin,
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
					advance_active_skill::<
						Queue,
						TAnimationsPlugin::TAnimationDispatch,
						SkillExecuter,
						Virtual,
					>,
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
		players: Query<Entity, Added<Player>>,
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

impl<TAnimationsPlugin, TLifeCyclePlugin, TInteractionsPlugin, TShadersPlugin> Plugin
	for SkillsPlugin<TAnimationsPlugin, TLifeCyclePlugin, TInteractionsPlugin, TShadersPlugin>
where
	TAnimationsPlugin: Plugin + HasAnimationsDispatch,
	TLifeCyclePlugin: Plugin + HandlesLifetime,
	TInteractionsPlugin: Plugin + HandlesAllEffects,
	TShadersPlugin: Plugin + HandlesEffectShadingForAll,
{
	fn build(&self, app: &mut App) {
		self.skill_load(app);
		self.item_load(app);
		self.skill_slot_load(app);
		self.skill_execution(app);
	}
}
