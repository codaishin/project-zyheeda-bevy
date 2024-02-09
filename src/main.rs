use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use project_zyheeda::{
	behaviors::MovementMode,
	bundles::Loadout,
	components::{
		Animator,
		CamOrbit,
		ComboTreeTemplate,
		Handed,
		Inventory,
		InventoryKey,
		Item,
		ItemType,
		Plasma,
		Player,
		PlayerMovement,
		PlayerSkills,
		Projectile,
		Side,
		SideUnset,
		SimpleMovement,
		SlotKey,
		Swap,
		Track,
		UnitsPerSecond,
		VoidSphere,
	},
	errors::Error,
	plugins::ingame_menu::IngameMenuPlugin,
	resources::{skill_templates::SkillTemplates, Models, MouseHover, Shared, SkillIcons, SlotMap},
	skill::{Active, Cast, Skill, SkillComboNext, SkillComboTree, SwordStrike},
	states::{GameRunning, MouseContext},
	systems::{
		animations::{
			link_animator::link_animators_with_new_animation_players,
			load_animations::load_animations,
			play_animations::play_animations,
		},
		behavior::{projectile::projectile_behavior, void_sphere::void_sphere_behavior},
		input::{
			schedule_slots::schedule_slots,
			set_cam_ray::set_cam_ray,
			set_mouse_hover::set_mouse_hover,
		},
		interactions::destroy_on_collision::destroy_on_collision,
		items::{
			equip::equip_item,
			slots::add_item_slots,
			swap::{equipped_items::swap_equipped_items, inventory_items::swap_inventory_items},
		},
		log::log_many,
		mouse_context::{
			advance::{advance_just_released_mouse_context, advance_just_triggered_mouse_context},
			release::release_triggered_mouse_context,
			trigger_primed::trigger_primed_mouse_context,
		},
		movement::{
			execute_move::execute_move,
			follow::follow,
			move_on_orbit::move_on_orbit,
			toggle_walk_run::player_toggle_walk_run,
		},
		prefab::instantiate::instantiate,
		skill::{
			chain_combo_skills::chain_combo_skills,
			dequeue::dequeue,
			enqueue::enqueue,
			execute_skill::execute_skill,
		},
		void_sphere::ring_rotation::ring_rotation,
	},
	tools::Tools,
	traits::{
		behavior::GetBehaviorMeta,
		orbit::{Orbit, Vec2Radians},
		prefab::AssetKey,
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
		.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
		.add_state::<GameRunning>()
		.add_state::<MouseContext>()
		.init_resource::<Shared<AssetKey, Handle<Mesh>>>()
		.init_resource::<Shared<AssetKey, Handle<StandardMaterial>>>()
		.add_systems(OnEnter(GameRunning::On), pause_virtual_time::<false>)
		.add_systems(OnExit(GameRunning::On), pause_virtual_time::<true>)
		.add_systems(
			PreStartup,
			(
				load_animations::<PlayerSkills<Side>, AssetServer>,
				load_animations::<PlayerMovement, AssetServer>,
			),
		)
		.add_systems(PreStartup, setup_skill_templates.pipe(log_many))
		.add_systems(Startup, setup_input)
		.add_systems(Startup, load_models)
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(
			First,
			(set_cam_ray::<Tools>, set_mouse_hover::<RapierContext>).chain(),
		)
		.add_systems(
			PreUpdate,
			(link_animators_with_new_animation_players, add_item_slots),
		)
		.add_systems(
			First,
			(
				schedule_slots::<KeyCode, Player, Input<KeyCode>, Input<KeyCode>>,
				schedule_slots::<KeyCode, Player, State<MouseContext>, Input<KeyCode>>,
				schedule_slots::<MouseButton, Player, Input<MouseButton>, Input<KeyCode>>
					.run_if(in_state(MouseContext::<KeyCode>::Default)),
			)
				.run_if(in_state(GameRunning::On)),
		)
		.add_systems(
			PreUpdate,
			(
				player_toggle_walk_run,
				enqueue::<MouseHover>,
				dequeue, // sets skill activity marker, so it MUST run before skill execution systems
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
				execute_skill::<
					PlayerSkills<Side>,
					Track<Skill<PlayerSkills<SideUnset>, Active>>,
					Virtual,
				>,
				execute_move::<PlayerMovement, Player, SimpleMovement, Virtual>,
			)
				.chain(),
		)
		.add_systems(
			Update,
			(
				instantiate::<Projectile<Plasma>>.pipe(log_many),
				projectile_behavior::<Projectile<Plasma>>,
				execute_move::<(), Projectile<Plasma>, SimpleMovement, Virtual>,
			)
				.chain(),
		)
		.add_systems(
			Update,
			(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>)
				.run_if(in_state(GameRunning::On)),
		)
		.add_systems(
			Update,
			(
				play_animations::<PlayerMovement, AnimationPlayer>,
				play_animations::<PlayerSkills<Side>, AnimationPlayer>,
			)
				.chain(),
		)
		.add_systems(
			Update,
			(
				instantiate::<VoidSphere>.pipe(log_many),
				ring_rotation,
				void_sphere_behavior,
			),
		)
		.add_systems(PostUpdate, destroy_on_collision);
}

#[cfg(debug_assertions)]
pub mod debug_utils {
	use super::*;
	use bevy::ecs::{archetype::Archetypes, component::Components, entity::Entities};
	use std::ops::Not;

	pub fn prepare_debug(app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.insert_resource(ShowGizmos::No)
			.add_systems(Update, debug)
			.add_systems(Update, toggle_gizmos)
			.add_systems(
				Update,
				forward_gizmo(&["projectile_spawn", "Player"], &Color::BLUE),
			)
			.add_systems(Update, display_events);
	}

	fn display_events(
		mut collision_events: EventReader<CollisionEvent>,
		mut contact_force_events: EventReader<ContactForceEvent>,
	) {
		for collision_event in collision_events.read() {
			println!("Received collision event: {:?}", collision_event);
		}

		for contact_force_event in contact_force_events.read() {
			println!("Received contact force event: {:?}", contact_force_event);
		}
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
		[("pistol", "pistol.gltf", 0), ("sword", "sword.gltf", 0)],
		&asset_server,
	);
	commands.insert_resource(models);
}

fn setup_input(mut commands: Commands) {
	commands.insert_resource(SlotMap::<MouseButton>::new([(
		MouseButton::Left,
		SlotKey::Legs,
		"Mouse Left",
	)]));
	commands.insert_resource(SlotMap::<KeyCode>::new([
		(KeyCode::E, SlotKey::Hand(Side::Main), "E"),
		(KeyCode::Q, SlotKey::Hand(Side::Off), "Q"),
	]));
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
	spawn_void_spheres(&mut commands);
	next_state.set(GameRunning::On);
}

fn setup_skill_templates(
	mut commands: Commands,
	assert_server: Res<AssetServer>,
) -> Vec<Result<(), Error>> {
	let (templates, errors) = SkillTemplates::new(&[
		Skill {
			name: "Swing Sword",
			cast: Cast {
				aim: Duration::ZERO,
				pre: Duration::from_millis(0),
				active: Duration::from_millis(500),
				after: Duration::from_millis(200),
			},
			soft_override: true,
			animate: PlayerSkills::SwordStrike(SideUnset),
			behavior: SwordStrike::behavior(),
			is_usable_with: HashSet::from([ItemType::Sword]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
				..default()
			},
			soft_override: true,
			animate: PlayerSkills::Shoot(Handed::Single(SideUnset)),
			behavior: Projectile::<Plasma>::behavior(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
		Skill {
			name: "Shoot Hand Gun Dual",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
				..default()
			},
			soft_override: true,
			animate: PlayerSkills::Shoot(Handed::Dual(SideUnset)),
			behavior: Projectile::<Plasma>::behavior(),
			is_usable_with: HashSet::from([ItemType::Pistol]),
			..default()
		},
		Skill {
			name: "Simple Movement",
			cast: Cast {
				aim: Duration::ZERO,
				after: Duration::MAX,
				..default()
			},
			behavior: SimpleMovement::behavior(),
			is_usable_with: HashSet::from([ItemType::Legs]),
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
		Name::from("Player"),
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
			camera: Camera {
				hdr: true,
				..default()
			},
			tonemapping: Tonemapping::TonyMcMapface,
			transform,
			..default()
		},
		BloomSettings::default(),
		orbit,
	));
}

fn spawn_void_spheres(commands: &mut Commands) {
	commands.spawn((
		Name::new("Sphere A"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(2., 0., 2.)),
	));
	commands.spawn((
		Name::new("Sphere B"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(-2., 0., 2.)),
	));
	commands.spawn((
		Name::new("Sphere C"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(2., 0., -2.)),
	));
	commands.spawn((
		Name::new("Sphere D"),
		VoidSphere,
		SpatialBundle::from_transform(Transform::from_xyz(-2., 0., -2.)),
	));
}
