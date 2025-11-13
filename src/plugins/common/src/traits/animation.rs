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
		+ for<'c> GetContextMut<Animations, TContext<'c>: OverrideAnimations>
		+ for<'c> GetContextMut<Animations, TContext<'c>: SetMovementDirection>;
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

pub trait SetMovementDirection {
	fn set_movement_direction(&mut self, direction: Dir3);
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

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Serialize, Deserialize)]
pub enum PlayMode {
	#[default]
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

#[derive(Debug, PartialEq, Default, Clone)]
pub struct AffectedAnimationBones2 {
	pub from_root: BoneName,
	pub until_exclusive: Vec<BoneName>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct BoneName(pub String);

impl From<&str> for BoneName {
	fn from(value: &str) -> Self {
		Self(String::from(value))
	}
}

impl From<&Name> for BoneName {
	fn from(value: &Name) -> Self {
		Self(value.to_string())
	}
}

impl PartialEq<Name> for BoneName {
	fn eq(&self, other: &Name) -> bool {
		self.0.as_str() == other.as_str()
	}
}

impl PartialEq<BoneName> for Name {
	fn eq(&self, other: &BoneName) -> bool {
		self.as_str() == other.0.as_str()
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum AnimationKey {
	Idle,
	Walk,
	Run,
	Skill(SlotKey),
}

pub enum MovementDirection {
	ToPoint(Vec3),
	Direction(Dir3),
}

impl From<Vec3> for MovementDirection {
	fn from(value: Vec3) -> Self {
		Self::ToPoint(value)
	}
}

impl From<Dir3> for MovementDirection {
	fn from(value: Dir3) -> Self {
		Self::Direction(value)
	}
}
