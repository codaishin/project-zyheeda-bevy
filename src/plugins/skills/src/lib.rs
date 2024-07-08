pub mod components;
pub mod items;
pub mod resources;
pub mod skills;
pub mod traits;

mod bundles;
mod skill_loader;
mod systems;

use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::{
	app::{Plugin, PreStartup, PreUpdate, Update},
	asset::{AssetApp, AssetServer, LoadedFolder},
	ecs::{
		entity::Entity,
		query::Added,
		schedule::{common_conditions::in_state, IntoSystemConfigs, State},
		system::{Commands, IntoSystem, Query, Res},
	},
	input::{keyboard::KeyCode, ButtonInput},
	time::Virtual,
};
use bundles::Loadout;
use common::{
	components::{Player, Side, Swap},
	resources::{key_map::KeyMap, Models},
	states::{GameRunning, MouseContext},
	systems::log::log_many,
	traits::{load_asset::Path, try_insert_on::TryInsertOn},
};
use components::{
	combo_node::ComboNode,
	combos::Combos,
	combos_time_out::CombosTimeOut,
	inventory::Inventory,
	queue::Queue,
	skill_executer::SkillExecuter,
	slots::Slots,
	Mounts,
};
use items::{inventory_key::InventoryKey, slot_key::SlotKey, Item, ItemType, Mount, SkillHandle};
use skill_loader::SkillLoader;
use skills::{QueuedSkill, Skill};
use std::{collections::HashSet, time::Duration};
use systems::{
	advance_active_skill::advance_active_skill,
	enqueue::enqueue,
	equip::equip_item,
	execute::execute,
	flush::flush,
	get_inputs::get_inputs,
	load_skills::load_skills,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	skill_spawn::add_skill_spawn,
	slots::add_item_slots,
	update_item_slot_skills::update_item_slot_skills,
	update_skill_combos::update_skill_combos,
};

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.init_resource::<KeyMap<SlotKey, KeyCode>>()
			.init_asset::<Skill>()
			.register_asset_loader(SkillLoader::<Skill>::default())
			.add_systems(PreStartup, load_skills::<AssetServer>)
			.add_systems(PreStartup, load_models)
			.add_systems(
				PreUpdate,
				(
					add_item_slots,
					add_skill_spawn,
					update_item_slot_skills::<LoadedFolder>,
				),
			)
			.add_systems(
				Update,
				(
					get_inputs::<
						KeyMap<SlotKey, KeyCode>,
						ButtonInput<KeyCode>,
						State<MouseContext<KeyCode>>,
					>
						.pipe(enqueue::<Slots, Skill, Queue, QueuedSkill>),
					update_skill_combos::<Combos, CombosTimeOut, Queue, Virtual>,
					advance_active_skill::<
						Queue,
						Animation,
						AnimationDispatch,
						SkillExecuter,
						Virtual,
					>,
					execute::<SkillExecuter>,
					flush::<Queue>,
				)
					.chain()
					.run_if(in_state(GameRunning::On)),
			)
			.add_systems(Update, set_player_items)
			.add_systems(
				Update,
				(
					trigger_primed_mouse_context,
					advance_just_triggered_mouse_context,
					release_triggered_mouse_context,
					advance_just_released_mouse_context,
				),
			)
			.add_systems(
				Update,
				(
					equip_item::<Player, (SlotKey, Option<Item>)>.pipe(log_many),
					equip_item::<Inventory, Swap<InventoryKey, SlotKey>>.pipe(log_many),
					equip_item::<Inventory, Swap<SlotKey, InventoryKey>>.pipe(log_many),
				),
			);
	}
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

	commands.try_insert_on(
		player,
		(
			SkillExecuter::default(),
			CombosTimeOut::after(Duration::from_secs(2)),
			get_inventory(),
			get_loadout(),
			get_combos(),
		),
	);
}

fn get_loadout() -> Loadout {
	Loadout::new(
		"projectile_spawn",
		[
			(
				SlotKey::Hand(Side::Off),
				Mounts {
					hand: "hand_slot.L",
					forearm: "lower_arm.L",
				},
			),
			(
				SlotKey::Hand(Side::Main),
				Mounts {
					hand: "hand_slot.R",
					forearm: "lower_arm.R",
				},
			),
		],
		[
			(
				SlotKey::Hand(Side::Off),
				Some(Item {
					name: "Plasma Pistol A",
					model: Some("pistol"),
					skill: SkillHandle::Path("skills/shoot_hand_gun.skill".into()),
					item_type: HashSet::from([ItemType::Pistol]),
					mount: Mount::Hand,
				}),
			),
			(
				SlotKey::Hand(Side::Main),
				Some(Item {
					name: "Force Bracer",
					model: Some("bracer"),
					skill: SkillHandle::Path("skills/force_shield.skill".into()),
					item_type: HashSet::from([ItemType::Bracer]),
					mount: Mount::Forearm,
				}),
			),
		],
	)
}

fn get_inventory() -> Inventory {
	Inventory::new([Some(Item {
		name: "Plasma Pistol B",
		model: Some("pistol"),
		skill: SkillHandle::Path("skills/shoot_hand_gun.skill".into()),
		item_type: HashSet::from([ItemType::Pistol]),
		mount: Mount::Hand,
	})])
}

fn get_combos() -> ComboNode<Path> {
	ComboNode::new([
		(
			SlotKey::Hand(Side::Off),
			(
				Path::from("skills/force_shield.skill"),
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Path::from("skills/gravity_well.skill"),
						ComboNode::default(),
					),
				)]),
			),
		),
		(
			SlotKey::Hand(Side::Main),
			(
				Path::from("skills/force_shield.skill"),
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Path::from("skills/gravity_well.skill"),
						ComboNode::default(),
					),
				)]),
			),
		),
	])
}
