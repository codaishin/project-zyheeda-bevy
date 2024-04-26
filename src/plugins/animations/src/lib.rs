pub mod animation;
pub mod components;

mod animation_keys;
mod resource;
mod systems;
mod traits;

use animation_keys::PlayerIdle;
use behaviors::components::{Movement, VelocityBased};
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PostUpdate, PreStartup, PreUpdate, Update},
	asset::AssetServer,
	ecs::system::IntoSystem,
};
use common::components::{Player, Side};
use components::PlayerMovement;
use skills::skill::PlayerSkills;
use systems::{
	active_animation::active_animation,
	idle_animation::idle_animation,
	link_animator::link_animators_with_new_animation_players,
	load_animations::load_animations,
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
				load_animations::<PlayerIdle, AssetServer>,
			),
		)
		.add_systems(PreUpdate, link_animators_with_new_animation_players)
		.add_systems(
			Update,
			set_movement_animation::<Player, Movement<VelocityBased>, PlayerMovement>,
		)
		.add_systems(
			PostUpdate,
			start
				.pipe(active_animation::<Player, PlayerSkills<Side>, AnimationPlayer>)
				.pipe(active_animation::<Player, PlayerMovement, AnimationPlayer>)
				.pipe(idle_animation::<Player, PlayerIdle, AnimationPlayer>),
		);
	}
}

fn start<T: Default>() -> T {
	T::default()
}
