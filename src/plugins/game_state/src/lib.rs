mod systems;

use behaviors::{
	components::cam_orbit::{CamOrbit, CamOrbitCenter},
	traits::{Orbit, Vec2Radians},
};
use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	components::MainCamera,
	states::{game_state::GameState, load_state::LoadState},
	traits::{
		register_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			RegisterLoadTracking,
			SetState,
		},
		try_insert_on::TryInsertOn,
	},
};
use enemy::components::void_sphere::VoidSphere;
use player::bundle::PlayerBundle;
use std::{f32::consts::PI, marker::PhantomData};
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> GameStatePlugin<TLoading>
where
	TLoading: Plugin + RegisterLoadTracking,
{
	pub fn depends_on(_: &TLoading) -> Self {
		GameStatePlugin(PhantomData)
	}
}

impl<TLoading> Plugin for GameStatePlugin<TLoading>
where
	TLoading: Plugin + RegisterLoadTracking,
{
	fn build(&self, app: &mut App) {
		let start_menu = GameState::StartMenu;
		let new_game = GameState::NewGame;
		let load_assets = GameState::Loading(LoadState::Assets);
		let load_dependencies = GameState::Loading(LoadState::Dependencies);
		let play = GameState::Play;

		TLoading::when_done::<AssetsProgress>().set_state(app, load_dependencies);
		TLoading::when_done::<DependenciesProgress>().set_state(app, play);

		app.insert_state(start_menu)
			.add_systems(PostStartup, spawn_camera)
			.add_systems(
				OnEnter(new_game),
				(setup_scene, transition_to_state(load_assets)).chain(),
			)
			.add_systems(OnEnter(play), pause_virtual_time::<false>)
			.add_systems(OnExit(play), pause_virtual_time::<true>);
	}
}

fn spawn_camera(mut commands: Commands) {
	commands.spawn((
		MainCamera,
		Camera3dBundle {
			camera: Camera {
				hdr: true,
				..default()
			},
			tonemapping: Tonemapping::TonyMcMapface,
			..default()
		},
		BloomSettings::default(),
	));
}

fn setup_scene(mut commands: Commands, cameras: Query<Entity, With<MainCamera>>) {
	let player = spawn_player(&mut commands);
	set_camera_to_orbit_player(&mut commands, cameras, player);
	spawn_void_spheres(&mut commands);
}

fn transition_to_state<TState: FreelyMutableState + Copy>(
	state: TState,
) -> impl Fn(ResMut<NextState<TState>>) {
	move |mut next_state: ResMut<NextState<TState>>| {
		next_state.set(state);
	}
}

fn spawn_player(commands: &mut Commands) -> Entity {
	commands.spawn(PlayerBundle::default()).id()
}

fn set_camera_to_orbit_player(
	commands: &mut Commands,
	cameras: Query<Entity, With<MainCamera>>,
	player: Entity,
) {
	for entity in &cameras {
		let mut transform = Transform::from_translation(Vec3::X);
		let mut orbit = CamOrbit {
			center: CamOrbitCenter::from(Vec3::ZERO).with_entity(player),
			distance: 15.,
			sensitivity: 1.,
		};

		orbit.orbit(&mut transform, Vec2Radians::new(-PI / 3., PI / 3.));
		orbit.sensitivity = 0.005;

		commands.try_insert_on(entity, (transform, orbit));
	}
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
