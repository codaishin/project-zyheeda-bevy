use bevy::{
	ecs::{archetype::Archetypes, component::Components, entity::Entities},
	prelude::*,
};
use project_zyheeda::{
	behavior::{Behavior, Idle, MovementMode, Run, SimpleMovement, Walk},
	components::{Animator, CamOrbit, Player, Queue, UnitsPerSecond},
	events::{Enqueue, MoveEvent},
	resources::Animation,
	systems::{
		animations::{animate::animate, link_animator::link_animators_with_new_animation_players},
		behavior::{dequeue::dequeue, player::schedule::player_enqueue},
		events::mouse_left::mouse_left,
		movement::{
			execute::execute,
			follow::follow,
			move_on_orbit::move_on_orbit,
			toggle_walk_run::player_toggle_walk_run,
		},
	},
	tools::Tools,
	traits::orbit::{Orbit, Vec2Radians},
};
use std::f32::consts::PI;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_event::<MoveEvent>()
		.add_event::<Enqueue<MoveEvent>>()
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(Update, link_animators_with_new_animation_players)
		.add_systems(
			Update,
			(mouse_left::<Tools, MoveEvent>, player_toggle_walk_run),
		)
		.add_systems(Update, player_enqueue::<MoveEvent, Behavior>)
		.add_systems(Update, dequeue::<Player, Behavior, SimpleMovement>)
		.add_systems(Update, (execute::<Player, Behavior, SimpleMovement>,))
		.add_systems(
			Update,
			(
				animate::<Player, Idle>,
				animate::<Player, Walk>,
				animate::<Player, Run>,
			),
		)
		.add_systems(Update, follow::<Player, CamOrbit>)
		.add_systems(Update, move_on_orbit::<CamOrbit>)
		.add_systems(Update, debug)
		.run();
}

fn debug(
	keyboard: Res<Input<KeyCode>>,
	all_entities: Query<Entity>,
	entities: &Entities,
	archetypes: &Archetypes,
	components: &Components,
) {
	if !keyboard.just_pressed(KeyCode::F12) {
		return;
	}
	for entity in all_entities.iter() {
		println!("Entity: {:?}", entity);
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
	commands.insert_resource(Animation::<Player, Idle>::new(
		asset_server.load("models/player.gltf#Animation2"),
	));
	commands.insert_resource(Animation::<Player, Walk>::new(
		asset_server.load("models/player.gltf#Animation1"),
	));
	commands.insert_resource(Animation::<Player, Run>::new(
		asset_server.load("models/player.gltf#Animation3"),
	));

	commands.spawn((
		SceneBundle {
			scene: asset_server.load("models/player.gltf#Scene0"),
			..default()
		},
		Player {
			walk_speed: UnitsPerSecond::new(0.75),
			run_speed: UnitsPerSecond::new(1.5),
			movement_mode: MovementMode::Walk,
		},
		Walk,
		Queue::<Behavior>::new(),
		Animator { ..default() },
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
		distance: 5.,
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
