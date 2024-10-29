mod systems;

use behaviors::{
	components::cam_orbit::{CamOrbit, CamOrbitCenter},
	traits::{Orbit, Vec2Radians},
};
use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use common::{components::MainCamera, states::GameState};
use enemy::components::void_sphere::VoidSphere;
use player::bundle::PlayerBundle;
use std::f32::consts::PI;
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostStartup, setup_simple_3d_scene)
			.add_systems(OnEnter(GameState::Play), pause_virtual_time::<false>)
			.add_systems(OnExit(GameState::Play), pause_virtual_time::<true>);
	}
}

fn setup_simple_3d_scene(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
	let player = spawn_player(&mut commands);
	spawn_camera(&mut commands, player);
	spawn_void_spheres(&mut commands);
	next_state.set(GameState::Play);
}

fn spawn_player(commands: &mut Commands) -> Entity {
	commands.spawn(PlayerBundle::default()).id()
}

fn spawn_camera(commands: &mut Commands, player: Entity) {
	let mut transform = Transform::from_translation(Vec3::X);
	let mut orbit = CamOrbit {
		center: CamOrbitCenter::from(Vec3::ZERO).with_entity(player),
		distance: 15.,
		sensitivity: 1.,
	};

	orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
	orbit.sensitivity = 0.005;

	commands.spawn((
		MainCamera,
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
	let directions = [
		("Sphere A", Vec3::new(1., 0., 1.)),
		("Sphere B", Vec3::new(-1., 0., 1.)),
		("Sphere C", Vec3::new(1., 0., -1.)),
		("Sphere D", Vec3::new(-1., 0., -1.)),
	];
	let distance = 10.;

	for (name, direction) in directions {
		commands.spawn((
			Name::new(name),
			VoidSphere,
			SpatialBundle::from_transform(Transform::from_translation(direction * distance)),
		));
	}
}
