pub mod animation;
pub mod components;
pub mod traits;

mod animation_keys;
mod resource;
mod systems;

use animation::Animation;
use animation_keys::PlayerIdle;
use behaviors::components::{Movement, VelocityBased};
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PostUpdate, PreStartup, Update},
	asset::AssetServer,
	ecs::{schedule::IntoSystemConfigs, system::IntoSystem},
	utils::Uuid,
};
use common::components::{Player, Side};
use components::{animation_dispatch::AnimationDispatch, PlayerMovement};
use resource::AnimationClips;
use skills::skill::PlayerSkills;
use systems::{
	active_animation::active_animation,
	idle_animation::idle_animation,
	link_animator::link_animators_with_new_animation_players,
	load_animation_clip::load_animation_clip,
	load_animations::load_animations,
	play_animation_clip::play_animation_clip,
	set_movement_animation::set_movement_animation,
};

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<AnimationClips<Uuid>>()
			.add_systems(
				PreStartup,
				(
					load_animations::<PlayerMovement, AssetServer>,
					load_animations::<PlayerSkills<Side>, AssetServer>,
					load_animations::<PlayerIdle, AssetServer>,
				),
			)
			.add_systems(
				Update,
				set_movement_animation::<Player, Movement<VelocityBased>, PlayerMovement>,
			)
			.add_systems(
				Update,
				(
					link_animators_with_new_animation_players,
					load_animation_clip::<Animation, AnimationDispatch, AssetServer>,
					play_animation_clip::<Animation, AnimationDispatch, AnimationPlayer>,
				)
					.chain(),
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
