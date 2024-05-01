pub mod components;
pub mod resources;
pub mod skill;
pub mod traits;

mod bundles;
mod systems;

use animations::{animation::Animation, components::animation_dispatch::AnimationDispatch};
use behaviors::components::{Plasma, Projectile};
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
	prelude::default,
	time::Virtual,
};
use bundles::Loadout;
use common::{
	components::{Player, Side, Swap},
	errors::Error,
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
use resources::{skill_templates::SkillTemplates, SkillIcons, SlotMap};
use skill::{Cast, Queued, ShootHandGun, Skill, SwordStrike};
use std::{
	collections::{HashMap, HashSet},
	time::Duration,
};
use systems::{
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
	update_active_skill::update_active_skill,
	update_skill_combos::update_skill_combos,
};
use traits::{GetExecution, GetSkillAnimation};

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(PreStartup, setup_skill_templates.pipe(log_many))
			.add_systems(PreStartup, load_models)
			.add_systems(PreStartup, setup_input)
			.add_systems(PreUpdate, add_item_slots)
			.add_systems(
				Update,
				(
					get_inputs::<ButtonInput<KeyCode>, State<MouseContext<KeyCode>>>
						.pipe(enqueue::<Slots, Skill, Queue, Skill<Queued>>),
					update_skill_combos::<Combos, Queue>,
					update_active_skill::<Queue, Animation, AnimationDispatch, Virtual>,
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

fn setup_skill_templates(
	mut commands: Commands,
	assert_server: Res<AssetServer>,
) -> Vec<Result<(), Error>> {
	let (templates, errors) = SkillTemplates::new(&[
		Skill {
			name: "Swing Sword",
			cast: Cast {
				pre: Duration::from_millis(0),
				active: Duration::from_millis(500),
				after: Duration::from_millis(200),
			},
			animate: Some(SwordStrike::animation()),
			execution: SwordStrike::execution(),
			is_usable_with: HashSet::from([ItemType::Sword]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
			},
			animate: Some(ShootHandGun::animation()),
			execution: Projectile::<Plasma>::execution(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
	]);
	let skill_icons = SkillIcons(HashMap::from([
		("Swing Sword", assert_server.load("icons/sword_down.png")),
		("Shoot Hand Gun", assert_server.load("icons/pistol.png")),
	]));

	commands.insert_resource(templates);
	commands.insert_resource(skill_icons);

	errors
		.iter()
		.cloned()
		.map(Err)
		.collect::<Vec<Result<(), Error>>>()
}

fn set_player_items(
	mut commands: Commands,
	skill_templates: Res<SkillTemplates>,
	players: Query<Entity, Added<Player>>,
) {
	let Ok(player) = players.get_single() else {
		return;
	};

	let Some(loadout) = get_loadout(&skill_templates) else {
		return;
	};

	let Some(inventory) = get_inventory(skill_templates) else {
		return;
	};

	commands.try_insert_on(player, (inventory, loadout));
}

fn get_loadout(skill_templates: &Res<'_, SkillTemplates>) -> Option<Loadout> {
	let shoot_hand_gun = skill_templates.get("Shoot Hand Gun")?;
	let slot_bones = [
		(SlotKey::SkillSpawn, "projectile_spawn"),
		(SlotKey::Hand(Side::Off), "hand_slot.L"),
		(SlotKey::Hand(Side::Main), "hand_slot.R"),
	];
	let equipment = [
		(
			SlotKey::Hand(Side::Off),
			Some(Item {
				name: "Pistol A",
				model: Some("pistol"),
				skill: Some(shoot_hand_gun.clone()),
				item_type: HashSet::from([ItemType::Pistol]),
			}),
		),
		(
			SlotKey::Hand(Side::Main),
			Some(Item {
				name: "Pistol B",
				model: Some("pistol"),
				skill: Some(shoot_hand_gun.clone()),
				item_type: HashSet::from([ItemType::Pistol]),
			}),
		),
	];

	Some(Loadout::new(slot_bones, equipment))
}

fn get_inventory(skill_templates: Res<'_, SkillTemplates>) -> Option<Inventory> {
	let shoot_hand_gun = skill_templates.get("Shoot Hand Gun")?;
	let swing_sword = skill_templates.get("Swing Sword")?;

	Some(Inventory::new([
		Some(Item {
			name: "Sword A",
			model: Some("sword"),
			skill: Some(swing_sword.clone()),
			item_type: HashSet::from([ItemType::Sword]),
		}),
		Some(Item {
			name: "Sword B",
			model: Some("sword"),
			skill: Some(swing_sword.clone()),
			item_type: HashSet::from([ItemType::Sword]),
		}),
		Some(Item {
			name: "Pistol C",
			model: Some("pistol"),
			skill: Some(shoot_hand_gun.clone()),
			item_type: HashSet::from([ItemType::Pistol]),
		}),
	]))
}
