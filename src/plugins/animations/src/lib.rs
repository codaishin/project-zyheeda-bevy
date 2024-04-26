pub mod animation;
pub mod components;
pub mod traits;

mod animation_keys;
mod resource;
mod systems;

use animation::Animation;
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, Update},
	asset::AssetServer,
	ecs::schedule::IntoSystemConfigs,
	utils::Uuid,
};
use components::animation_dispatch::AnimationDispatch;
use resource::AnimationClips;
use systems::{
	link_animator::link_animators_with_new_animation_players,
	load_animation_clip::load_animation_clip,
	play_animation_clip::play_animation_clip,
};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<AnimationClips<Uuid>>().add_systems(
			Update,
			(
				link_animators_with_new_animation_players,
				load_animation_clip::<Animation, AnimationDispatch, AssetServer>,
				play_animation_clip::<Animation, AnimationDispatch, AnimationPlayer>,
			)
				.chain(),
		);
	}
}
