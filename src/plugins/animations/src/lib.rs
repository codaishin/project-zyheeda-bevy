pub mod components;
mod resource;
mod systems;
mod traits;

use behaviors::components::SimpleMovement;
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PreStartup, PreUpdate, Update},
	asset::AssetServer,
	ecs::system::{In, IntoSystem},
};
use common::components::{Player, Side};
use components::PlayerMovement;
use skills::skill::PlayerSkills;
use systems::{
	link_animator::link_animators_with_new_animation_players,
	load_animations::load_animations,
	play_animations::play_animations,
	set_movement_animation::set_movement_animation,
};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			PreStartup,
			(
				load_animations::<PlayerMovement, AssetServer>,
				load_animations::<PlayerSkills<Side>, AssetServer>,
			),
		)
		.add_systems(PreUpdate, link_animators_with_new_animation_players)
		.add_systems(
			Update,
			set_movement_animation::<Player, SimpleMovement, PlayerMovement>,
		)
		.add_systems(
			Update,
			start
				.pipe(play_animations::<PlayerSkills<Side>, AnimationPlayer>)
				.pipe(play_animations::<PlayerMovement, AnimationPlayer>)
				.pipe(drop),
		);
	}
}

fn start<T: Default>() -> T {
	T::default()
}

fn drop<T>(_: In<T>) {}
