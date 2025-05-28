pub mod components;
pub mod item;
pub mod resources;
pub mod skills;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;
mod tools;

use bevy::prelude::*;
use bundles::{ComboBundle, Loadout};
use common::{
	states::game_state::{GameState, LoadingGame},
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
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
		handles_settings::HandlesSettings,
		handles_skill_behaviors::HandlesSkillBehaviors,
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
	skill_spawners::SkillSpawners,
	slots::{ForearmItemSlots, HandItemSlots, Slots, SubMeshEssenceSlots},
	swapper::Swapper,
};
use item::{Item, dto::ItemDto};
use macros::item_asset;
use skills::{QueuedSkill, RunSkillBehavior, Skill, dto::SkillDto};
use std::{marker::PhantomData, time::Duration};
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
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
	TPlayers: ThreadSafe
		+ HandlesPlayer
		+ HandlesPlayerCameras
		+ HandlesPlayerMouse
		+ ConfiguresPlayerSkillAnimations,
	TMenu: ThreadSafe + HandlesLoadoutMenu + HandlesComboMenu,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
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

		app.add_systems(
			PreUpdate,
			SkillSpawners::track_in_self_and_children::<Name>().system(),
		)
		.add_systems(Update, Self::set_player_items)
		.add_systems(Update, Swapper::system);
	}

	fn skill_execution(&self, app: &mut App) {
		let execute_skill = SkillExecuter::<RunSkillBehavior>::execute_system::<
			TLifeCycles,
			TInteractions,
			TBehaviors,
			TPlayers,
		>;

		app.register_required_components::<TBehaviors::TSkillContact, Transform>();
		app.register_required_components::<TBehaviors::TSkillContact, Visibility>();
		app.add_systems(
			Update,
			(
				get_inputs::<TSettings::TKeyMap<SlotKey>, ButtonInput<UserInput>>
					.pipe(enqueue::<Slots, Queue, QueuedSkill>),
				Combos::update::<Queue>,
				flush_skill_combos::<Combos, CombosTimeOut, Virtual, Queue>,
				advance_active_skill::<Queue, TPlayers, TBehaviors, SkillExecuter, Virtual>,
				execute_skill.pipe(log_many),
				flush::<Queue>,
			)
				.chain()
				.before(TBehaviors::SKILL_BEHAVIOR_SYSTEMS)
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
	TLifeCycles: ThreadSafe + HandlesLifetime,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TDispatchChildrenAssets: ThreadSafe + HandlesAssetsForChildren,
	TLoading: ThreadSafe + HandlesCustomAssets + HandlesCustomFolderAssets,
	TSettings: ThreadSafe + HandlesSettings,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors + HandlesOrientation,
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
