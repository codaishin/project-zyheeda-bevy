use bevy::prelude::*;
use project_zyheeda::{
	behaviors::MovementMode,
	bundles::Loadout,
	components::{
		Animator,
		CamOrbit,
		ComboTreeTemplate,
		Inventory,
		InventoryKey,
		Item,
		ItemType,
		Marker,
		Player,
		Projectile,
		Side,
		SimpleMovement,
		SlotKey,
		Swap,
		Track,
		UnitsPerSecond,
	},
	errors::Error,
	markers::{
		functions::{insert_hand_marker_fn, remove_hand_marker_fn},
		meta::MarkerMeta,
		Dual,
		Fast,
		HandGun,
		Idle,
		Left,
		Right,
		Slow,
		Sword,
	},
	plugins::ingame_menu::IngameMenuPlugin,
	resources::{skill_templates::SkillTemplates, Animation, Models, SlotMap},
	skill::{Active, Cast, Skill, SkillComboNext, SkillComboTree},
	states::GameRunning,
	systems::{
		animations::{animate::animate, link_animator::link_animators_with_new_animation_players},
		input::schedule_slots::schedule_slots,
		items::{
			equip::equip_item,
			slots::add_item_slots,
			swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
		},
		log::{log, log_many},
		movement::{
			execute_move::execute_move,
			follow::follow,
			move_on_orbit::move_on_orbit,
			toggle_walk_run::player_toggle_walk_run,
		},
		skill::{
			chain_combo_skills::chain_combo_skills,
			dequeue::dequeue,
			enqueue::enqueue,
			execute_skill::execute_skill,
			projectile::projectile,
		},
	},
	tools::{Once, Repeat, Tools},
	traits::{
		behavior::GetBehaviorMeta,
		marker::GetMarkerHandMarkerMeta,
		orbit::{Orbit, Vec2Radians},
	},
};
use std::{
	collections::{HashMap, HashSet},
	f32::consts::PI,
	time::Duration,
};

fn main() {
	let app = &mut App::new();

	prepare_game(app);

	#[cfg(debug_assertions)]
	debug_utils::prepare_debug(app);

	app.run();
}

fn prepare_game(app: &mut App) {
	app.add_plugins(DefaultPlugins)
		.add_plugins(IngameMenuPlugin)
		.add_state::<GameRunning>()
		.add_systems(OnEnter(GameRunning::On), pause_virtual_time::<false>)
		.add_systems(OnExit(GameRunning::On), pause_virtual_time::<true>)
		.add_systems(PreStartup, setup_skill_templates.pipe(log_many))
		.add_systems(Startup, setup_input)
		.add_systems(Startup, load_models)
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(
			PreUpdate,
			(
				link_animators_with_new_animation_players,
				add_item_slots,
				player_toggle_walk_run,
				schedule_slots::<MouseButton, Player>,
				schedule_slots::<KeyCode, Player>,
				enqueue::<Tools>,
				dequeue, // sets skill activity marker, so it MUST run before skill execution systems
			)
				.run_if(in_state(GameRunning::On)),
		)
		.add_systems(
			Update,
			(
				equip_item::<Player, (SlotKey, Option<Item>)>.pipe(log_many),
				equip_item::<Inventory, Swap<InventoryKey, SlotKey>>.pipe(log_many),
				equip_item::<Inventory, Swap<SlotKey, InventoryKey>>.pipe(log_many),
				swap_equipped_items.pipe(log_many),
				swap_inventory_items,
			),
		)
		.add_systems(
			Update,
			(
				chain_combo_skills::<SkillComboNext>,
				execute_skill::<Track<Skill<Active>>, Virtual>.pipe(log_many),
				execute_move::<Player, SimpleMovement, Virtual>,
			),
		)
		.add_systems(
			Update,
			(
				projectile.pipe(log),
				execute_move::<Projectile, SimpleMovement, Virtual>,
			),
		)
		.add_systems(
			Update,
			(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>)
				.run_if(in_state(GameRunning::On)),
		)
		.add_systems(
			Update,
			(
				animate::<Player, Marker<Idle>, Repeat>,
				animate::<Player, Marker<Slow>, Repeat>,
				animate::<Player, Marker<Fast>, Repeat>,
				animate::<Player, Marker<(HandGun, Left)>, Once>,
				animate::<Player, Marker<(HandGun, Left)>, Once>,
				animate::<Player, Marker<(HandGun, Left, Dual)>, Once>,
				animate::<Player, Marker<(HandGun, Right)>, Once>,
				animate::<Player, Marker<(HandGun, Right, Dual)>, Once>,
				animate::<Player, Marker<(Sword, Left)>, Once>,
				animate::<Player, Marker<(Sword, Right)>, Once>,
			),
		);
}

#[cfg(debug_assertions)]
pub mod debug_utils {
	use super::*;
	use bevy::ecs::{archetype::Archetypes, component::Components, entity::Entities};
	use std::ops::Not;

	pub fn prepare_debug(app: &mut App) {
		app.insert_resource(ShowGizmos::No)
			.add_systems(Update, debug)
			.add_systems(Update, toggle_gizmos)
			.add_systems(Update, forward_gizmo(&["projectile_spawn"], &Color::BLUE));
	}

	fn debug(
		keyboard: Res<Input<KeyCode>>,
		all_entities: Query<Entity>,
		names: Query<&Name>,
		entities: &Entities,
		archetypes: &Archetypes,
		components: &Components,
	) {
		if !keyboard.just_pressed(KeyCode::F12) {
			return;
		}
		for entity in all_entities.iter() {
			println!("Entity: {:?}", entity);
			let name = names.get(entity);
			println!(
				"Entity (Name): {}",
				name.map(|n| n.as_str()).unwrap_or("No Name")
			);
			let Some(entity_location) = entities.get(entity) else {
				return;
			};
			let Some(archetype) = archetypes.get(entity_location.archetype_id) else {
				return;
			};
			for component in archetype.components() {
				if let Some(info) = components.get_info(component) {
					println!("\tComponent: {}", info.name());
				}
			}
		}
	}

	#[derive(Resource, PartialEq, Clone, Copy)]
	enum ShowGizmos {
		Yes,
		No,
	}

	impl Not for ShowGizmos {
		type Output = ShowGizmos;

		fn not(self) -> Self::Output {
			match self {
				ShowGizmos::Yes => ShowGizmos::No,
				ShowGizmos::No => ShowGizmos::Yes,
			}
		}
	}

	fn toggle_gizmos(mut show_gizmos: ResMut<ShowGizmos>, keys: Res<Input<KeyCode>>) {
		if keys.just_pressed(KeyCode::F11) {
			*show_gizmos = !*show_gizmos;
		}
	}

	fn forward_gizmo<const N: usize>(
		targets: &'static [&str; N],
		color: &'static Color,
	) -> impl Fn(Gizmos, Query<(&Name, &GlobalTransform)>, Res<ShowGizmos>) {
		|mut gizmos, agents, show_gizmos| {
			if *show_gizmos == ShowGizmos::No {
				return;
			}

			for (name, transform) in &agents {
				if targets.contains(&name.as_str()) {
					gizmos.ray(transform.translation(), transform.forward() * 10., *color);
				}
			}
		}
	}
}

fn pause_virtual_time<const PAUSE: bool>(mut time: ResMut<Time<Virtual>>) {
	if PAUSE {
		time.pause();
	} else {
		time.unpause();
	}
}

fn load_models(mut commands: Commands, asset_server: Res<AssetServer>) {
	let models = Models::new(
		[
			("pistol", "pistol.gltf", 0),
			("sword", "sword.gltf", 0),
			("projectile", "projectile.gltf", 0),
		],
		&asset_server,
	);
	commands.insert_resource(models);
}

fn setup_input(mut commands: Commands) {
	commands.insert_resource(SlotMap::<MouseButton>(
		[(MouseButton::Left, SlotKey::Legs)].into(),
	));
	commands.insert_resource(SlotMap::<KeyCode>(
		[
			(KeyCode::E, SlotKey::Hand(Side::Main)),
			(KeyCode::Q, SlotKey::Hand(Side::Off)),
		]
		.into(),
	));
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	skill_templates: Res<SkillTemplates>,
	mut next_state: ResMut<NextState<GameRunning>>,
) {
	spawn_plane(&mut commands, &mut meshes, &mut materials);
	spawn_player(&mut commands, asset_server, skill_templates);
	spawn_light(&mut commands);
	spawn_camera(&mut commands);
	next_state.set(GameRunning::On);
}

fn setup_skill_templates(mut commands: Commands) -> Vec<Result<(), Error>> {
	let (templates, errors) = SkillTemplates::new(&[
		Skill {
			name: "Swing Sword",
			cast: Cast {
				pre: Duration::from_millis(0),
				active: Duration::from_millis(500),
				after: Duration::from_millis(200),
			},
			soft_override: true,
			marker: Sword::hand_markers(),
			behavior: Sword::behavior(),
			is_usable_with: HashSet::from([ItemType::Sword]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun",
			cast: Cast {
				pre: Duration::from_millis(500),
				active: Duration::ZERO,
				after: Duration::from_millis(500),
			},
			soft_override: true,
			marker: HandGun::hand_markers(),
			behavior: Projectile::behavior(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun Dual",
			cast: Cast {
				pre: Duration::from_millis(500),
				active: Duration::ZERO,
				after: Duration::from_millis(500),
			},
			soft_override: true,
			marker: MarkerMeta {
				insert_fn: insert_hand_marker_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>,
				remove_fn: remove_hand_marker_fn::<(HandGun, Left, Dual), (HandGun, Right, Dual)>,
			},
			behavior: Projectile::behavior(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
		Skill {
			name: "Simple Movement",
			cast: Cast {
				after: Duration::MAX,
				..default()
			},
			behavior: SimpleMovement::behavior(),
			is_usable_with: HashSet::from([ItemType::Legs]),
			..default()
		},
	]);

	commands.insert_resource(templates);

	errors
		.iter()
		.cloned()
		.map(Err)
		.collect::<Vec<Result<(), Error>>>()
}

fn spawn_plane(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn(PbrBundle {
		mesh: meshes.add(shape::Plane::from_size(5.0).into()),
		material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
		..default()
	});
}

fn spawn_player(
	commands: &mut Commands,
	asset_server: Res<AssetServer>,
	skill_templates: Res<SkillTemplates>,
) {
	commands.insert_resource(Animation::<Player, Marker<Idle>>::new(
		asset_server.load("models/player.gltf#Animation2"),
	));
	commands.insert_resource(Animation::<Player, Marker<Slow>>::new(
		asset_server.load("models/player.gltf#Animation1"),
	));
	commands.insert_resource(Animation::<Player, Marker<Fast>>::new(
		asset_server.load("models/player.gltf#Animation3"),
	));
	commands.insert_resource(Animation::<Player, Marker<(HandGun, Right)>>::new(
		asset_server.load("models/player.gltf#Animation4"),
	));
	commands.insert_resource(Animation::<Player, Marker<(HandGun, Left)>>::new(
		asset_server.load("models/player.gltf#Animation5"),
	));
	commands.insert_resource(Animation::<Player, Marker<(HandGun, Right, Dual)>>::new(
		asset_server.load("models/player.gltf#Animation6"),
	));
	commands.insert_resource(Animation::<Player, Marker<(HandGun, Left, Dual)>>::new(
		asset_server.load("models/player.gltf#Animation7"),
	));
	commands.insert_resource(Animation::<Player, Marker<(Sword, Right)>>::new(
		asset_server.load("models/player.gltf#Animation8"),
	));
	commands.insert_resource(Animation::<Player, Marker<(Sword, Left)>>::new(
		asset_server.load("models/player.gltf#Animation9"),
	));

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
	let legs = Item {
		name: "Legs",
		model: None,
		skill: skill_templates.get("Simple Movement").cloned(),
		item_type: HashSet::from([ItemType::Legs]),
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

	commands.spawn((
		SceneBundle {
			scene: asset_server.load("models/player.gltf#Scene0"),
			..default()
		},
		Animator { ..default() },
		Player {
			walk_speed: UnitsPerSecond::new(0.75),
			run_speed: UnitsPerSecond::new(1.5),
			movement_mode: MovementMode::Slow,
		},
		Inventory::new([Some(sword_a), Some(sword_b), Some(pistol_c)]),
		Loadout::new(
			[
				(SlotKey::SkillSpawn, "projectile_spawn"),
				(SlotKey::Hand(Side::Off), "hand_slot.L"),
				(SlotKey::Hand(Side::Main), "hand_slot.R"),
				(SlotKey::Legs, "root"), // FIXME: using root as placeholder for now
			],
			[
				(SlotKey::Hand(Side::Off), pistol_a.into()),
				(SlotKey::Hand(Side::Main), pistol_b.into()),
				(SlotKey::Legs, legs.into()),
			],
		),
		skill_combos,
	));
}

fn spawn_light(commands: &mut Commands) {
	commands.spawn(PointLightBundle {
		point_light: PointLight {
			intensity: 1500.0,
			shadows_enabled: true,
			..default()
		},
		transform: Transform::from_xyz(4.0, 8.0, 4.0),
		..default()
	});
}

fn spawn_camera(commands: &mut Commands) {
	let mut transform = Transform::from_translation(Vec3::X);
	let mut orbit = CamOrbit {
		center: Vec3::ZERO,
		distance: 10.,
		sensitivity: 1.,
	};

	orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
	orbit.sensitivity = 0.005;

	commands.spawn((
		Camera3dBundle {
			transform,
			..default()
		},
		orbit,
	));
}
