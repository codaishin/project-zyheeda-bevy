use super::iteration::IterFinite;
use crate::{
	tools::{action_key::slot::SlotKey, path::Path},
	traits::accessors::get::GetContextMut,
};
use bevy::{
	ecs::{component::Mutable, system::SystemParam},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait HandlesAnimations {
	type TAnimationsMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<Animations, TContext<'c>: RegisterAnimations2>
		+ for<'c> GetContextMut<Animations, TContext<'c>: OverrideAnimations>;
}

pub struct Animations {
	pub entity: Entity,
}

pub type AnimationsParamMut<'w, 's, T> = <T as HandlesAnimations>::TAnimationsMut<'w, 's>;

pub trait RegisterAnimations2 {
	fn register_animations(&mut self, animations: HashMap<AnimationKey, Animation2>);
}

pub trait OverrideAnimations {
	fn override_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority> + 'static,
		TAnimations: IntoIterator<Item = AnimationKey> + 'static;
}

pub trait StartAnimation {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait SetAnimations {
	fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
	where
		TLayer: Into<AnimationPriority> + 'static,
		TAnimations: IntoIterator<Item = Animation> + 'static;
}

pub trait StopAnimation {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: Into<AnimationPriority> + 'static;
}

pub trait GetAnimationDefinitions
where
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AffectedAnimationBones: From<&'a Self::TAnimationMask>,
{
	type TAnimationMask: IterFinite;

	fn animations() -> HashMap<AnimationPath, AnimationMask>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum AffectedAnimationBones {
	SubTree {
		root: Name,
		until_exclusive: Vec<Name>,
	},
	Leaf {
		root: Name,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AnimationPath {
	Single(Path),
	Directional(Directional),
}

impl From<&'static str> for AnimationPath {
	fn from(path: &'static str) -> Self {
		Self::Single(Path::from(path))
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Directional {
	pub forward: Path,
	pub backward: Path,
	pub left: Path,
	pub right: Path,
}

pub trait HasAnimationsDispatch {
	type TAnimationDispatch: StartAnimation
		+ SetAnimations
		+ StopAnimation
		+ Component<Mutability = Mutable>;
}

pub trait ConfigureNewAnimationDispatch {
	fn configure_animation_dispatch(
		&self,
		new_animation_dispatch: &mut (impl StartAnimation + StopAnimation),
	);
}

pub trait GetMovementDirection {
	fn movement_direction(&self, transform: &GlobalTransform) -> Option<Dir3>;
}

pub trait RegisterAnimations: HasAnimationsDispatch {
	fn register_animations<TAgent>(app: &mut App)
	where
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AffectedAnimationBones: From<&'a TAgent::TAnimationMask>;

	fn register_movement_direction<TMovementDirection>(app: &mut App)
	where
		TMovementDirection: Component + GetMovementDirection;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AnimationPriority {
	High,
	Medium,
	Low,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum PlayMode {
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Animation {
	pub path: AnimationPath,
	pub play_mode: PlayMode,
}

impl Animation {
	pub const fn new(path: AnimationPath, play_mode: PlayMode) -> Self {
		Self { path, play_mode }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Animation2 {
	pub path: AnimationPath,
	pub play_mode: PlayMode,
	pub mask: AnimationMask,
	pub bones: AffectedAnimationBones2,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AffectedAnimationBones2 {
	from_root: BoneName,
	until_exclusive: Vec<BoneName>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BoneName(pub String);

pub enum AnimationKey {
	Idle,
	Walk,
	Run,
	Skill(SlotKey),
}
