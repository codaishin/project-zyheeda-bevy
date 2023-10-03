#[cfg(test)]
mod test_tools;

mod behavior;
mod components;
mod events;
mod resources;
mod systems;
mod tools;
mod traits;

use behavior::{Idle, SimpleMovement};
use bevy::ecs::{archetype::Archetypes, component::Components, entity::Entities};
use bevy::prelude::*;
use components::{Behaviors, CamOrbit, Player, UnitsPerSecond};
use events::{MoveEnqueueEvent, MoveEvent};
use resources::PlayerAnimations;
use std::f32::consts::PI;
use systems::{
	animations::animate,
	clean::clean,
	events::mouse_left::mouse_left,
	helpers::add_player_animator::add_player_animator,
	movement::{execute::execute, follow::follow, move_on_orbit::move_on_orbit},
	player_behavior::schedule::schedule,
};
use tools::Tools;
use traits::{
	new::New,
	orbit::{Orbit, Vec2Radians},
};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_event::<MoveEvent>()
		.add_event::<MoveEnqueueEvent>()
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(Update, add_player_animator)
		.add_systems(Update, mouse_left::<Tools, MoveEvent, MoveEnqueueEvent>)
		.add_systems(
			Update,
			schedule::<MoveEvent, MoveEnqueueEvent, SimpleMovement, Behaviors>,
		)
		.add_systems(Update, execute::<SimpleMovement, Behaviors>)
		.add_systems(Update, follow::<Player, CamOrbit>)
		.add_systems(Update, move_on_orbit::<CamOrbit>)
		.add_systems(
			Update,
			(
				animate::<SimpleMovement, Behaviors, PlayerAnimations>,
				animate::<Idle, Behaviors, PlayerAnimations>,
			),
		)
		.add_systems(Update, clean::<Behaviors>)
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
	commands.insert_resource(PlayerAnimations {
		idle: asset_server.load("models/player.gltf#Animation0"),
		walk: asset_server.load("models/player.gltf#Animation1"),
	});

	commands.spawn((
		SceneBundle {
			scene: asset_server.load("models/player.gltf#Scene0"),
			..default()
		},
		Player {
			movement_speed: UnitsPerSecond::new(0.75),
		},
		Behaviors::new(),
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
