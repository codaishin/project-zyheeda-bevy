use super::load_asset::Path;
use crate::{
	systems::init_associated_component::GetAssociated,
	tools::{Last, This},
};
use bevy::prelude::*;

pub enum AnimationPriority {
	High,
	Medium,
	Low,
}

pub trait StartAnimation {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
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

pub trait AnimationDispatchType {
	type AnimationDispatch;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlayMode {
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Animation {
	pub path: Path,
	pub play_mode: PlayMode,
	pub update_fn: Option<fn(This<Animation>, Last<Animation>)>,
}

impl Animation {
	pub fn new(path: Path, play_mode: PlayMode) -> Self {
		Self {
			path,
			play_mode,
			update_fn: None,
		}
	}
}
