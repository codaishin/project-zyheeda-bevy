use bevy::{
	ecs::{archetype::Archetypes, component::Components, entity::Entities},
	prelude::*,
};
use project_zyheeda::{
	behaviors::{movement::movement, shoot_gun::shoot_gun, MovementMode},
	bundles::Loadout,
	components::{
		marker::{Fast, HandGun, Idle, Marker, Right, Shoot, Slow},
		Animator,
		CamOrbit,
		Equip,
		Item,
		Player,
		Projectile,
		Side,
		SimpleMovement,
		SlotBones,
		SlotKey,
		UnitsPerSecond,
	},
	resources::{Animation, Models, SlotMap},
	systems::{
		animations::{animate::animate, link_animator::link_animators_with_new_animation_players},
		behavior::{dequeue::dequeue, enqueue::enqueue, projectile::projectile},
		input::schedule_slots::schedule_slots,
		items::{equip::equip_items, slots::add_item_slots},
		log::log,
		movement::{
			execute_move::execute_move,
			follow::follow,
			move_on_orbit::move_on_orbit,
			toggle_walk_run::player_toggle_walk_run,
		},
		skill::execute_skill,
	},
	tools::Tools,
	traits::orbit::{Orbit, Vec2Radians},
};
use std::f32::consts::PI;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_systems(Startup, setup_input)
		.add_systems(Startup, load_models)
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(PreUpdate, link_animators_with_new_animation_players)
		.add_systems(PreUpdate, add_item_slots)
		.add_systems(Update, equip_items)
		.add_systems(
			Update,
			(
				player_toggle_walk_run,
				schedule_slots::<MouseButton, Player>,
				schedule_slots::<KeyCode, Player>,
				enqueue::<Tools>,
				dequeue,
			),
		)
		.add_systems(
			Update,
			(execute_skill, execute_move::<Player, SimpleMovement>),
		)
		.add_systems(
			Update,
			(
				projectile.pipe(log),
				execute_move::<Projectile, SimpleMovement>,
			),
		)
		.add_systems(
			Update,
			(
				animate::<Player, Marker<Idle>>,
				animate::<Player, Marker<Slow>>,
				animate::<Player, Marker<Fast>>,
				animate::<Player, Marker<(Shoot, HandGun, Right)>>,
			),
		)
		.add_systems(
			Update,
			(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>),
		)
		.add_systems(Update, debug)
		.run();
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

fn load_models(mut commands: Commands, asset_server: Res<AssetServer>) {
	let models = Models::new(
		[
			("pistol", "pistol.gltf", 0),
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
		[(KeyCode::E, SlotKey::Hand(Side::Right))].into(),
	));
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	spawn_plane(&mut commands, &mut meshes, &mut materials);
	spawn_player(&mut commands, asset_server);
	spawn_light(&mut commands);
	spawn_camera(&mut commands);
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

fn spawn_player(commands: &mut Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(Animation::<Player, Marker<Idle>>::new(
		asset_server.load("models/player.gltf#Animation2"),
	));
	commands.insert_resource(Animation::<Player, Marker<Slow>>::new(
		asset_server.load("models/player.gltf#Animation1"),
	));
	commands.insert_resource(Animation::<Player, Marker<Fast>>::new(
		asset_server.load("models/player.gltf#Animation3"),
	));
	commands.insert_resource(Animation::<Player, Marker<(Shoot, HandGun, Right)>>::new(
		asset_server.load("models/player.gltf#Animation4"),
	));

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
		Loadout::new(
			SlotBones(
				[
					(SlotKey::SkillSpawn, "projectile_spawn"),
					(SlotKey::Hand(Side::Right), "hand_slot.R"),
					(SlotKey::Legs, "root"), // FIXME: using root as placeholder for now
				]
				.into(),
			),
			Equip::new([
				Item {
					slot: SlotKey::Hand(Side::Right),
					model: Some("pistol"),
					behavior: Some(shoot_gun()),
				},
				Item {
					slot: SlotKey::Legs,
					model: None,
					behavior: Some(movement()),
				},
			]),
		),
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
