mod systems;

use bevy::{
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
	prelude::*,
};
use common::{
	components::MainCamera,
	states::{game_state::GameState, transition_to_state},
	traits::handles_load_tracking::{HandlesLoadTracking, OnLoadingDone},
};
use enemy::components::void_sphere::VoidSphere;
use player::bundle::PlayerBundle;
use std::marker::PhantomData;
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> GameStatePlugin<TLoading>
where
	TLoading: Plugin + HandlesLoadTracking,
{
	pub fn depends_on(_: &TLoading) -> Self {
		GameStatePlugin(PhantomData)
	}
}

impl<TLoading> Plugin for GameStatePlugin<TLoading>
where
	TLoading: Plugin + HandlesLoadTracking,
{
	fn build(&self, app: &mut App) {
		let start_menu = GameState::StartMenu;
		let new_game = GameState::NewGame;
		let loading = GameState::Loading;
		let play = GameState::Play;

		TLoading::begin_loading_on(app, loading).when_done_set(play);

		app.insert_state(start_menu)
			.add_systems(PostStartup, spawn_camera)
			.add_systems(
				OnEnter(new_game),
				(setup_scene, transition_to_state(loading)).chain(),
			)
			.add_systems(OnEnter(play), pause_virtual_time::<false>)
			.add_systems(OnExit(play), pause_virtual_time::<true>);
	}
}

fn spawn_camera(mut commands: Commands) {
	commands.spawn((
		MainCamera,
		Camera3d::default(),
		Camera {
			hdr: true,
			..default()
		},
		Tonemapping::TonyMcMapface,
		Bloom::default(),
	));
}

fn setup_scene(mut commands: Commands) {
	spawn_player(&mut commands);
	spawn_void_spheres(&mut commands);
}

fn spawn_player(commands: &mut Commands) -> Entity {
	commands.spawn(PlayerBundle::default()).id()
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
			Transform::from_translation(direction * distance),
			Visibility::default(),
		));
	}
}
