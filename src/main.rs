#[cfg(test)]
mod test_tools;

mod components;
mod events;
mod systems;
mod tools;
mod traits;

use bevy::prelude::*;
use components::{CamOrbit, Player, SimpleMovement, UnitsPerSecond};
use events::MouseEvent;
use std::f32::consts::PI;
use systems::{
	events::send_move_command::send_move_command,
	movement::{move_on_orbit::move_on_orbit, move_player::move_player},
};
use tools::Tools;
use traits::orbit::{Orbit, Vec2Radians};

fn main() {
	App::new()
		.add_plugins(DefaultPlugins)
		.add_event::<MouseEvent>()
		.add_systems(Startup, setup_simple_3d_scene)
		.add_systems(Update, send_move_command::<MouseEvent, Tools>)
		.add_systems(Update, move_player::<MouseEvent, SimpleMovement>)
		.add_systems(Update, move_on_orbit::<CamOrbit>)
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
		SimpleMovement {
			target: None,
			speed: UnitsPerSecond::new(1.),
		},
		Player,
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
