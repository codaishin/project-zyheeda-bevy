mod behaviors;
mod components;
mod item;
mod resources;
mod skills;
mod systems;
mod tools;
mod traits;

use crate::components::{
	combos::dto::CombosDto,
	combos_time_out::dto::CombosTimeOutDto,
	loadout::Loadout,
	queue::dto::QueueDto,
	skill_executer::dto::SkillExecuterDto,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingGame},
	tools::action_key::{
		slot::{Side, SlotKey},
		user_input::UserInput,
	},
	traits::{
		handles_assets_for_children::HandlesAssetsForChildren,
		handles_combo_menu::{ConfigureCombos, HandlesComboMenu},
		handles_custom_assets::{HandlesCustomAssets, HandlesCustomFolderAssets},
		handles_effect::HandlesAllEffects,
		handles_lifetime::HandlesLifetime,
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
		try_insert_on::TryInsertOn,
	},
};
use components::{
	combo_node::ComboNode,
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::{ForearmItemSlots, HandItemSlots, Slots, SubMeshEssenceSlots},
	swapper::Swapper,
};
use item::{Item, dto::ItemDto};
use macros::item_asset;
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

impl<
	TSaveGame,
	TLifeCycles,
	TInteractions,
	TDispatchChildrenAssets,
	TLoading,
	TSettings,
	TBehaviors,
	TPlayers,
	TMenu,
>
	SkillsPlugin<(
		TSaveGame,
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
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
		_: &TLifeCycles,
		_: &TInteractions,
		_: &TDispatchChildrenAssets,
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
		TDispatchChildrenAssets::register_child_asset::<Slots, HandItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, ForearmItemSlots>(app);
		TDispatchChildrenAssets::register_child_asset::<Slots, SubMeshEssenceSlots>(app);

		app.register_required_components::<TPlayers::TPlayer, Loadout>()
			.add_systems(Update, Self::set_player_items)
			.add_systems(Update, Swapper::system);
	}

	fn skill_execution(&self, app: &mut App) {
		TSaveGame::register_savable_component::<CombosTimeOut>(app);
		TSaveGame::register_savable_component::<Combos>(app);
		TSaveGame::register_savable_component::<Queue>(app);
		TSaveGame::register_savable_component::<SkillExecuter>(app);

		let get_inputs = get_inputs::<TSettings::TKeyMap<SlotKey>, ButtonInput<UserInput>>;
		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TLifeCycles,
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
				advance_active_skill::<Queue, TPlayers, TBehaviors, SkillExecuter, Virtual>,
				execute_skill,
				flush::<Queue>,
			)
				.chain()
				.before(TBehaviors::SYSTEMS)
				.run_if(in_state(GameState::Play)),
		);
	}

	fn set_player_items(
		mut commands: Commands,
		players: Query<Entity, Added<TPlayers::TPlayer>>,
		asset_server: Res<AssetServer>,
	) {
		let Ok(player) = players.single() else {
			return;
		};
		let asset_server = asset_server.as_ref();

		commands.try_insert_on(
			player,
			(
				Swapper::default(),
				Self::get_inventory(asset_server),
				Self::get_slots(asset_server),
			),
		);
	}

	fn get_slots(asset_server: &AssetServer) -> Slots {
		Slots::new([
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

	fn config_menus(&self, app: &mut App) {
		TMenu::loadout_with_swapper::<Swapper>().configure(
			app,
			Inventory::describe_loadout_for::<TPlayers::TPlayer>,
			Slots::describe_loadout_for::<TPlayers::TPlayer>,
		);
		TMenu::configure_quickbar_menu(
			app,
			get_quickbar_descriptors_for::<TPlayers::TPlayer, Slots, Queue, Combos>,
		);
		TMenu::combos_with_skill::<Skill>().configure(
			app,
			ComboDescriptor::describe_combos_for::<TPlayers::TPlayer>,
			Combos::<ComboNode>::update_for::<TPlayers::TPlayer>,
		);
	}
}

impl<
	TSaveGame,
	TLifeCycles,
	TInteractions,
	TDispatchChildrenAssets,
	TLoading,
	TSettings,
	TBehaviors,
	TPlayers,
	TMenu,
> Plugin
	for SkillsPlugin<(
		TSaveGame,
		TLifeCycles,
		TInteractions,
		TDispatchChildrenAssets,
		TLoading,
		TSettings,
		TBehaviors,
		TPlayers,
		TMenu,
	)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
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
