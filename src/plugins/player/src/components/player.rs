use animations::{
	animation::{Animation, PlayMode},
	components::animation_dispatch::AnimationDispatch,
	traits::{GetAnimationPaths, IdleLayer, StartAnimation},
};
use bevy::prelude::*;
use common::{systems::init_associated_component::GetAssociated, traits::load_asset::Path};

#[derive(Component, Default, Debug, PartialEq)]
pub struct Player;

impl Player {
	pub const MODEL_PATH: &'static str = "models/player.glb";

	pub fn animation_path(animation_name: &str) -> Path {
		Path::from(Self::MODEL_PATH.to_owned() + "#" + animation_name)
	}
}

impl GetAnimationPaths for Player {
	fn animation_paths() -> Vec<Path> {
		vec![
			Player::animation_path("Animation0"),
			Player::animation_path("Animation1"),
			Player::animation_path("Animation2"),
			Player::animation_path("Animation3"),
			Player::animation_path("Animation4"),
			Player::animation_path("Animation5"),
			Player::animation_path("Animation6"),
			Player::animation_path("Animation7"),
		]
	}
}

impl GetAssociated<AnimationDispatch> for Player {
	fn get_associated_component() -> AnimationDispatch {
		let mut animation_dispatch = AnimationDispatch::default();
		animation_dispatch.start_animation(
			IdleLayer,
			Animation::new(Player::animation_path("Animation1"), PlayMode::Repeat),
		);

		animation_dispatch
	}
}
