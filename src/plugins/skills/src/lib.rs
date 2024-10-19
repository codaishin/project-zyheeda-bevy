pub mod components;
pub mod definitions;
pub mod items;
pub mod resources;
pub mod skills;
pub mod systems;
pub mod traits;

mod behaviors;
mod bundles;

use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::prelude::*;
use bundles::{ComboBundle, Loadout};
use common::{
	components::{Collection, Player, Side, Swap},
	resources::{key_map::KeyMap, Models},
	states::{GameRunning, MouseContext},
	systems::{log::log_many, track_components::TrackComponentInSelfAndChildren},
	traits::{register_folder_assets::RegisterFolderAssets, try_insert_on::TryInsertOn},
};
use components::{
	combo_node::ComboNode,
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	lookup::Lookup,
	queue::Queue,
	skill_executer::SkillExecuter,
	skill_spawners::SkillSpawners,
	slots::Slots,
};
use definitions::item_slots::{ForearmSlots, HandSlots};
use items::{inventory_key::InventoryKey, slot_key::SlotKey, Item, ItemType};
use skills::{skill_data::SkillData, QueuedSkill, RunSkillBehavior, Skill};
use std::time::Duration;
use systems::{
	advance_active_skill::advance_active_skill,
	enqueue::enqueue,
	equip::equip_item,
	execute::ExecuteSkills,
	flush::flush,
	get_inputs::get_inputs,
	load_models::{
		apply_load_models_commands::apply_load_models_commands,
		load_models_commands_for_new_slots::load_models_commands_for_new_slots,
	},
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	update_skill_combos::update_skill_combos,
	uuid_to_skill::uuid_to_skill,
};
use uuid::{uuid, Uuid};

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
	fn build(&self, app: &mut App) {
		skill_load(app);
		inventory(app);
		skill_combo_load(app);
		skill_slot_load(app);
		skill_execution(app);
	}
}

fn skill_load(app: &mut App) {
	app.register_folder_assets::<Skill, SkillData>();
}

fn inventory(app: &mut App) {
	app.add_systems(
		PreUpdate,
		uuid_to_skill::<Inventory<Uuid>, Inventory<Skill>>,
	);
}

fn skill_slot_load(app: &mut App) {
	type Hands = Lookup<HandSlots<Player>>;
	type Forearms = Lookup<ForearmSlots<Player>>;
	type SubModels = Lookup<ForearmSlots<Player>>;

	app.add_systems(Startup, load_models)
		.add_systems(
			PreUpdate,
			(
				SkillSpawners::track_in_self_and_children::<Name>().system(),
				load_models_commands_for_new_slots,
			),
		)
		.add_systems(PreUpdate, uuid_to_skill::<Slots<Uuid>, Slots>)
		.add_systems(Update, set_player_items)
		.add_systems(
			Update,
			(
				Hands::track_in_self_and_children::<Name>().system(),
				Forearms::track_in_self_and_children::<Name>().system(),
				SubModels::track_in_self_and_children::<Name>()
					.with::<Handle<Mesh>>()
					.system(),
			)
		)
		.add_systems(
			Update,
			(
				equip_item::<
					Inventory<Skill>,
					InventoryKey,
					Collection<Swap<InventoryKey, SlotKey>>,
				>
					.pipe(log_many),
				equip_item::<
					Inventory<Skill>,
					InventoryKey,
					Collection<Swap<SlotKey, InventoryKey>>,
				>
					.pipe(log_many),
				apply_load_models_commands::<Hands, Forearms>.pipe(log_many),
			),
		);
}

fn skill_execution(app: &mut App) {
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
				update_skill_combos::<Combos, CombosTimeOut, Queue, Virtual>,
				advance_active_skill::<Queue, Animation, AnimationDispatch, SkillExecuter, Virtual>,
				SkillExecuter::<RunSkillBehavior>::execute_system.pipe(log_many),
				flush::<Queue>,
			)
				.chain()
				.run_if(in_state(GameRunning::On)),
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

fn skill_combo_load(app: &mut App) {
	app.add_systems(PreUpdate, uuid_to_skill::<ComboNode<Uuid>, Combos>);
}

fn load_models(mut commands: Commands, asset_server: Res<AssetServer>) {
	let models = Models::new(
		[("pistol", "pistol.glb", 0), ("bracer", "bracer.glb", 0)],
		&asset_server,
	);
	commands.insert_resource(models);
}

fn set_player_items(mut commands: Commands, players: Query<Entity, Added<Player>>) {
	let Ok(player) = players.get_single() else {
		return;
	};

	commands.try_insert_on(player, (get_inventory(), get_loadout(), get_combos()));
}

fn get_loadout() -> Loadout<Player> {
	Loadout::new([
		(
			SlotKey::TopHand(Side::Left),
			Some(Item {
				name: "Plasma Pistol A",
				model: Some("pistol"),
				skill: Some(uuid!("b2d5b9cb-b09d-42d4-a0cc-556cb118ef2e")),
				item_type: ItemType::Pistol,
			}),
		),
		(
			SlotKey::BottomHand(Side::Left),
			Some(Item {
				name: "Plasma Pistol B",
				model: Some("pistol"),
				skill: Some(uuid!("b2d5b9cb-b09d-42d4-a0cc-556cb118ef2e")),
				item_type: ItemType::Pistol,
			}),
		),
		(
			SlotKey::BottomHand(Side::Right),
			Some(Item {
				name: "Force Bracer",
				model: Some("bracer"),
				skill: Some(uuid!("a27de679-0fab-4e21-b4f0-b5a6cddc6aba")),
				item_type: ItemType::Bracer,
			}),
		),
		(
			SlotKey::TopHand(Side::Right),
			Some(Item {
				name: "Force Bracer",
				model: Some("bracer"),
				skill: Some(uuid!("a27de679-0fab-4e21-b4f0-b5a6cddc6aba")),
				item_type: ItemType::Bracer,
			}),
		),
	])
}

fn get_inventory() -> Inventory<Uuid> {
	Inventory::new([
		Some(Item {
			name: "Plasma Pistol C",
			model: Some("pistol"),
			skill: Some(uuid!("b2d5b9cb-b09d-42d4-a0cc-556cb118ef2e")),
			item_type: ItemType::Pistol,
		}),
		Some(Item {
			name: "Plasma Pistol D",
			model: Some("pistol"),
			skill: Some(uuid!("b2d5b9cb-b09d-42d4-a0cc-556cb118ef2e")),
			item_type: ItemType::Pistol,
		}),
		Some(Item {
			name: "Plasma Pistol E",
			model: Some("pistol"),
			skill: Some(uuid!("b2d5b9cb-b09d-42d4-a0cc-556cb118ef2e")),
			item_type: ItemType::Pistol,
		}),
	])
}

fn get_combos() -> ComboBundle {
	let timeout = CombosTimeOut::after(Duration::from_secs(2));
	let combos = [];

	ComboBundle::with_timeout(timeout).with_predefined_combos(combos)
}
