use super::{iteration::IterFinite, load_asset::Path};
use bevy::prelude::*;
use std::collections::HashMap;

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

pub trait GetAnimationDefinitions
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskDefinition: From<&'a Self::TAnimationMask>,
{
	type TAnimationMask: IterFinite;

	fn animations() -> HashMap<Path, AnimationMask>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnimationMaskDefinition {
	Mask {
		from_root: Name,
		exclude_roots: Vec<Name>,
	},
	Leaf {
		from_root: Name,
	},
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
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskDefinition: From<&'a TAgent::TAnimationMask>;
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
}

impl Animation {
	pub fn new(path: Path, play_mode: PlayMode) -> Self {
		Self { path, play_mode }
	}
}
