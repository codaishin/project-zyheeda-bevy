#[cfg(test)]
mod test_tools;

mod behaviors;
mod components;
mod events;
mod systems;
mod tools;
mod traits;

use behaviors::SimpleMovement;
use bevy::prelude::*;
use components::{Behaviors, CamOrbit, Player, UnitsPerSecond};
use events::{MoveEnqueueEvent, MoveEvent};
use std::f32::consts::PI;
use systems::{
	clean::clean,
	events::mouse_left::mouse_left,
	movement::{execute::execute, move_on_orbit::move_on_orbit},
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
		.add_systems(Update, mouse_left::<Tools, MoveEvent, MoveEnqueueEvent>)
		.add_systems(Update, schedule::<MoveEvent, SimpleMovement, Behaviors>)
		.add_systems(Update, execute::<SimpleMovement, Behaviors>)
		.add_systems(Update, move_on_orbit::<CamOrbit>)
		.add_systems(Update, clean::<Behaviors>)
		.run();
}

fn setup_simple_3d_scene(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	spawn_plane(&mut commands, &mut meshes, &mut materials);
	spawn_cube(&mut commands, &mut meshes, &mut materials);
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

fn spawn_cube(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		PbrBundle {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
			transform: Transform::from_xyz(0.0, 0.5, 0.0),
			..default()
		},
		Player {
			movement_speed: UnitsPerSecond::new(1.),
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
