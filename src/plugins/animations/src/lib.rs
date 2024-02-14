pub mod components;
mod resource;
mod systems;
mod traits;

use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PreStartup, PreUpdate, Update},
	asset::AssetServer,
};
use common::{components::Side, skill::PlayerSkills};
use components::PlayerMovement;
use systems::{
	link_animator::link_animators_with_new_animation_players,
	load_animations::load_animations,
	play_animations::play_animations,
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
			(
				play_animations::<PlayerMovement, AnimationPlayer>,
				play_animations::<PlayerSkills<Side>, AnimationPlayer>,
			),
		);
	}
}
