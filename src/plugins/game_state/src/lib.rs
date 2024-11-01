pub mod states;

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
use common::{components::MainCamera, traits::try_insert_on::TryInsertOn};
use enemy::components::void_sphere::VoidSphere;
use player::bundle::PlayerBundle;
use states::{game_state::GameState, menu_state::MenuState};
use std::f32::consts::PI;
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin;

impl GameStatePlugin {
	pub const START: GameState = GameState::StartMenu;
	pub const NEW_GAME: GameState = GameState::NewGame;
	pub const LOADING: GameState = GameState::Loading;
	pub const PLAY: GameState = GameState::Play;
	pub const INVENTORY: GameState = GameState::IngameMenu(MenuState::Inventory);
	pub const COMBO_OVERVIEW: GameState = GameState::IngameMenu(MenuState::ComboOverview);
}

impl Plugin for GameStatePlugin {
	fn build(&self, app: &mut App) {
		app.insert_state(Self::START)
			.add_systems(PostStartup, spawn_camera)
			.add_systems(
				OnEnter(Self::NEW_GAME),
				setup_scene_and_set_state_to(Self::PLAY),
			)
			.add_systems(OnEnter(Self::PLAY), pause_virtual_time::<false>)
			.add_systems(OnExit(Self::PLAY), pause_virtual_time::<true>);
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

fn setup_scene_and_set_state_to<TState>(
	state: TState,
) -> impl Fn(Commands, ResMut<NextState<TState>>, Query<Entity, With<MainCamera>>)
where
	TState: FreelyMutableState + Copy,
{
	move |mut commands: Commands,
	      mut next_state: ResMut<NextState<TState>>,
	      cameras: Query<Entity, With<MainCamera>>| {
		let player = spawn_player(&mut commands);
		set_camera_to_orbit_player(&mut commands, cameras, player);
		spawn_void_spheres(&mut commands);

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
