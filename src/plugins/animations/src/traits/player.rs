use super::{GetAnimationPaths, IdleLayer, StartAnimation};
use crate::{
	animation::{Animation, PlayMode},
	components::animation_dispatch::AnimationDispatch,
};
use common::{
	components::Player,
	systems::init_associated_component::GetAssociated,
	tools::player_animation_path,
	traits::load_asset::Path,
};

impl GetAnimationPaths for Player {
	fn animation_paths() -> Vec<Path> {
		vec![
			player_animation_path("Animation0"),
			player_animation_path("Animation1"),
			player_animation_path("Animation2"),
			player_animation_path("Animation3"),
			player_animation_path("Animation4"),
			player_animation_path("Animation5"),
			player_animation_path("Animation6"),
			player_animation_path("Animation7"),
		]
	}
}

impl GetAssociated<AnimationDispatch> for Player {
	fn get_associated_component() -> AnimationDispatch {
		let mut animation_dispatch = AnimationDispatch::default();
		animation_dispatch.start_animation(
			IdleLayer,
			Animation::new(
				Path::from(Player::MODEL_PATH.to_owned() + "#Animation2"),
				PlayMode::Repeat,
			),
		);

		animation_dispatch
	}
}
