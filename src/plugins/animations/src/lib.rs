pub mod animation;
pub mod components;
pub mod traits;

mod animation_keys;
mod resource;
mod systems;

use animation::Animation;
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PostUpdate, Update},
	asset::AssetServer,
	ecs::schedule::IntoSystemConfigs,
};
use common::traits::load_asset::Path;
use components::animation_dispatch::AnimationDispatch;
use resource::AnimationClips;
use systems::{
	flush::flush,
	link_animator::link_animators_with_new_animation_players,
	load_animation_clip::load_animation_clip,
	play_animation_clip::play_animation_clip,
};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<AnimationClips<Path>>()
			.add_systems(
				Update,
				(
					link_animators_with_new_animation_players,
					load_animation_clip::<Animation, AnimationDispatch, AssetServer>,
					play_animation_clip::<Animation, AnimationDispatch, AnimationPlayer>,
				)
					.chain(),
			)
			.add_systems(PostUpdate, flush::<AnimationDispatch>);
	}
}
