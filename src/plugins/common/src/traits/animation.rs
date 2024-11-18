use bevy::prelude::Component;

use crate::systems::init_associated_component::GetAssociated;

use super::load_asset::Path;

pub enum AnimationPriority {
	High,
	Medium,
	Low,
}

pub trait StartAnimation<TAnimation> {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait StopAnimation {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait GetAnimationPaths {
	fn animation_paths() -> Vec<Path>;
}

pub trait RegisterAnimations<TAnimationDispatch>
where
	TAnimationDispatch: Component,
{
	fn register_animations<
		TAgent: Component + GetAnimationPaths + GetAssociated<TAnimationDispatch>,
	>(
		&mut self,
	);
}
