mod bundles;
pub mod components;
pub mod resources;
pub mod skill;
pub mod states;
mod systems;
pub mod traits;

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
	systems::log::log_many,
};
use components::{
	queue::{DequeueAble, EnqueueAble, QueueCollection},
	ComboTreeTemplate,
	Handed,
	Inventory,
	InventoryKey,
	Item,
	ItemType,
	SideUnset,
	SlotKey,
};
use resources::{skill_templates::SkillTemplates, SkillIcons, SlotMap};
use skill::{Cast, PlayerSkills, Skill, SkillComboNext, SkillComboTree, SwordStrike};
use states::{GameRunning, MouseContext};
use std::{
	collections::{HashMap, HashSet},
	time::Duration,
};
use systems::{
	chain_combo_skills::chain_combo_skills,
	dequeue::dequeue,
	equip::equip_item,
	get_inputs::get_inputs,
	mouse_context::{
		advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
		release::release_triggered_mouse_context,
		trigger_primed::trigger_primed_mouse_context,
	},
	queue::{
		set_queue_to_dequeue::set_queue_to_dequeue,
		set_queue_to_enqueue::set_queue_to_enqueue,
	},
	set_slot_visibility::set_slot_visibility,
	skill_activation::skill_activation,
	skill_activity_dispatch::skill_activity_dispatch,
	skill_execution::skill_execution,
	slots::add_item_slots,
};
use traits::GetExecution;

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.init_state::<MouseContext>()
			.add_systems(PreStartup, setup_skill_templates.pipe(log_many))
			.add_systems(PreStartup, load_models)
			.add_systems(PreStartup, setup_input)
			.add_systems(PreUpdate, add_item_slots)
			.add_systems(
				Update,
				(
					set_queue_to_enqueue,
					get_inputs::<ButtonInput<KeyCode>, State<MouseContext<KeyCode>>>
						.pipe(skill_activation::<QueueCollection<EnqueueAble>>)
						.pipe(log_many),
					chain_combo_skills::<SkillComboNext, QueueCollection<EnqueueAble>>,
					set_queue_to_dequeue,
					skill_activity_dispatch::<
						PlayerSkills<Side>,
						QueueCollection<DequeueAble>,
						Virtual,
					>
						.pipe(log_many),
					set_slot_visibility,
					skill_execution,
					dequeue::<QueueCollection<DequeueAble>>.pipe(log_many),
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
			animate: Some(PlayerSkills::SwordStrike(SideUnset)),
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
			animate: Some(PlayerSkills::Shoot(Handed::Single(SideUnset))),
			execution: Projectile::<Plasma>::execution(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun Dual",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
			},
			animate: Some(PlayerSkills::Shoot(Handed::Dual(SideUnset))),
			execution: Projectile::<Plasma>::execution(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			dual_wield: true,
			..default()
		},
	]);
	let skill_icons = SkillIcons(HashMap::from([
		("Swing Sword", assert_server.load("icons/sword_down.png")),
		("Shoot Hand Gun", assert_server.load("icons/pistol.png")),
		(
			"Shoot Hand Gun Dual",
			assert_server.load("icons/pistol_dual.png"),
		),
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

	let pistol_a = Item {
		name: "Pistol A",
		model: Some("pistol"),
		skill: skill_templates.get("Shoot Hand Gun").cloned(),
		item_type: HashSet::from([ItemType::Pistol]),
	};
	let pistol_b = Item {
		name: "Pistol B",
		model: Some("pistol"),
		skill: skill_templates.get("Shoot Hand Gun").cloned(),
		item_type: HashSet::from([ItemType::Pistol]),
	};
	let pistol_c = Item {
		name: "Pistol C",
		model: Some("pistol"),
		skill: skill_templates.get("Shoot Hand Gun").cloned(),
		item_type: HashSet::from([ItemType::Pistol]),
	};
	let sword_a = Item {
		name: "Sword A",
		model: Some("sword"),
		skill: skill_templates.get("Swing Sword").cloned(),
		item_type: HashSet::from([ItemType::Sword]),
	};
	let sword_b = Item {
		name: "Sword B",
		model: Some("sword"),
		skill: skill_templates.get("Swing Sword").cloned(),
		item_type: HashSet::from([ItemType::Sword]),
	};

	// FIXME: Use a more sensible pattern to register predefined combos
	let mut skill_combos = ComboTreeTemplate(default());
	let shoot_hand_gun = skill_templates.get("Shoot Hand Gun");
	let shoot_hand_gun_dual = skill_templates.get("Shoot Hand Gun Dual");
	if let (Some(shoot_hand_gun), Some(shoot_hand_gun_dual)) = (shoot_hand_gun, shoot_hand_gun_dual)
	{
		skill_combos.0 = HashMap::from([
			(
				SlotKey::Hand(Side::Main),
				SkillComboTree {
					skill: shoot_hand_gun.clone(),
					next: SkillComboNext::Tree(HashMap::from([(
						SlotKey::Hand(Side::Off),
						SkillComboTree {
							skill: shoot_hand_gun_dual.clone(),
							next: SkillComboNext::Alternate {
								slot_key: SlotKey::Hand(Side::Main),
								skill: shoot_hand_gun_dual.clone(),
							},
						},
					)])),
				},
			),
			(
				SlotKey::Hand(Side::Off),
				SkillComboTree {
					skill: shoot_hand_gun.clone(),
					next: SkillComboNext::Tree(HashMap::from([(
						SlotKey::Hand(Side::Main),
						SkillComboTree {
							skill: shoot_hand_gun_dual.clone(),
							next: SkillComboNext::Alternate {
								slot_key: SlotKey::Hand(Side::Off),
								skill: shoot_hand_gun_dual.clone(),
							},
						},
					)])),
				},
			),
		]);
	}

	let mut player = commands.entity(player);
	player.insert((
		Inventory::new([Some(sword_a), Some(sword_b), Some(pistol_c)]),
		Loadout::new(
			[
				(SlotKey::SkillSpawn, "projectile_spawn"),
				(SlotKey::Hand(Side::Off), "hand_slot.L"),
				(SlotKey::Hand(Side::Main), "hand_slot.R"),
			],
			[
				(SlotKey::Hand(Side::Off), pistol_a.into()),
				(SlotKey::Hand(Side::Main), pistol_b.into()),
			],
		),
		skill_combos,
	));
}
