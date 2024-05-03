pub mod components;
pub mod resources;
pub mod skills;
pub mod traits;

mod bundles;
mod systems;

use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use bevy::{
	app::{Plugin, PreStartup, PreUpdate, Update},
	asset::AssetServer,
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
	resources::Models,
	states::{GameRunning, MouseContext},
	systems::log::log_many,
	traits::try_insert_on::TryInsertOn,
};
use components::{
	combos::Combos,
	inventory::Inventory,
	queue::Queue,
	slots::Slots,
	InventoryKey,
	Item,
	ItemType,
	SlotKey,
};
use resources::{SkillIcons, SlotMap};
use skills::{shoot_hand_gun::ShootHandGun, Queued, Skill};
use std::collections::{HashMap, HashSet};
use systems::{
	advance_active_skill::advance_active_skill,
	apply_skill_behavior::apply_skill_behavior,
	enqueue::enqueue,
	equip::equip_item,
	flush::flush,
	get_inputs::get_inputs,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	slots::add_item_slots,
	update_skill_combos::update_skill_combos,
};
use traits::SkillTemplate;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(PreStartup, setup_skill_icons)
			.add_systems(PreStartup, load_models)
			.add_systems(PreStartup, setup_input)
			.add_systems(PreUpdate, add_item_slots)
			.add_systems(
				Update,
				(
					get_inputs::<ButtonInput<KeyCode>, State<MouseContext<KeyCode>>>
						.pipe(enqueue::<Slots, Skill, Queue, Skill<Queued>>),
					update_skill_combos::<Combos, Queue>,
					advance_active_skill::<Queue, Animation, AnimationDispatch, Virtual>,
					apply_skill_behavior,
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
		[("pistol", "pistol.gltf", 0), ("sword", "sword.gltf", 0)],
		&asset_server,
	);
	commands.insert_resource(models);
}

fn setup_input(mut commands: Commands) {
	commands.insert_resource(SlotMap::new([
		(KeyCode::KeyE, SlotKey::Hand(Side::Main), "E"),
		(KeyCode::KeyQ, SlotKey::Hand(Side::Off), "Q"),
	]));
}

fn setup_skill_icons(mut commands: Commands, assert_server: Res<AssetServer>) {
	let skill_icons = SkillIcons(HashMap::from([
		("Swing Sword", assert_server.load("icons/sword_down.png")),
		("Shoot Hand Gun", assert_server.load("icons/pistol.png")),
	]));

	commands.insert_resource(skill_icons);
}

fn set_player_items(mut commands: Commands, players: Query<Entity, Added<Player>>) {
	let Ok(player) = players.get_single() else {
		return;
	};

	commands.try_insert_on(player, (get_inventory(), get_loadout()));
}

fn get_loadout() -> Loadout {
	Loadout::new(
		[
			(SlotKey::SkillSpawn, "projectile_spawn"),
			(SlotKey::Hand(Side::Off), "hand_slot.L"),
			(SlotKey::Hand(Side::Main), "hand_slot.R"),
		],
		[
			(
				SlotKey::Hand(Side::Off),
				Some(Item {
					name: "Pistol A",
					model: Some("pistol"),
					skill: Some(ShootHandGun::skill()),
					item_type: HashSet::from([ItemType::Pistol]),
				}),
			),
			(
				SlotKey::Hand(Side::Main),
				Some(Item {
					name: "Pistol B",
					model: Some("pistol"),
					skill: Some(ShootHandGun::skill()),
					item_type: HashSet::from([ItemType::Pistol]),
				}),
			),
		],
	)
}

fn get_inventory() -> Inventory {
	Inventory::new([Some(Item {
		name: "Pistol C",
		model: Some("pistol"),
		skill: Some(ShootHandGun::skill()),
		item_type: HashSet::from([ItemType::Pistol]),
	})])
}
