use super::load_asset::Path;
use crate::tools::{Last, This};
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

pub trait GetAnimationDefinitions {
	type TAnimationMask: Into<AnimationMask>;

	fn animation_definitions() -> Vec<(Option<Self::TAnimationMask>, Path)>;
}

pub trait HasAnimationsDispatch {
	type TAnimationDispatch: StartAnimation + StopAnimation + Component;
}

pub trait ConfigureNewAnimationDispatch {
	fn configure_animation_dispatch(
		&self,
		new_animation_dispatch: &mut (impl StartAnimation + StopAnimation),
	);
}

pub trait RegisterAnimations: HasAnimationsDispatch {
	fn register_animations<TAgent>(app: &mut App)
	where
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch;
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
