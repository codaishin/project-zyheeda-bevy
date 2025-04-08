use super::{
	iteration::{Iter, IterFinite},
	load_asset::Path,
};
use bevy::prelude::*;
use std::{collections::HashMap, ops::Index};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

	fn animations() -> HashMap<AnimationAsset, AnimationMask>;
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AnimationAsset {
	Single(Path),
	Directional(DirectionPaths),
}

impl From<&'static str> for AnimationAsset {
	fn from(path: &'static str) -> Self {
		Self::Single(Path::from(path))
	}
}

impl From<Path> for AnimationAsset {
	fn from(path: Path) -> Self {
		Self::Single(path)
	}
}

impl From<Directions> for AnimationAsset {
	fn from(directions: Directions) -> Self {
		Self::Directional(DirectionPaths { directions })
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MovementDirection {
	Forward,
	Backward,
	Left,
	Right,
}

impl IterFinite for MovementDirection {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Forward))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match &current.0? {
			MovementDirection::Forward => Some(MovementDirection::Backward),
			MovementDirection::Backward => Some(MovementDirection::Left),
			MovementDirection::Left => Some(MovementDirection::Right),
			MovementDirection::Right => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DirectionPaths {
	directions: Directions,
}

impl DirectionPaths {
	fn get(&self, direction: &MovementDirection) -> &Path {
		match direction {
			MovementDirection::Forward => &self.directions.forward,
			MovementDirection::Backward => &self.directions.backward,
			MovementDirection::Left => &self.directions.left,
			MovementDirection::Right => &self.directions.right,
		}
	}
}

impl Index<&MovementDirection> for DirectionPaths {
	type Output = Path;

	fn index(&self, index: &MovementDirection) -> &Self::Output {
		self.get(index)
	}
}

impl Index<MovementDirection> for DirectionPaths {
	type Output = Path;

	fn index(&self, index: MovementDirection) -> &Self::Output {
		self.get(&index)
	}
}

impl From<Directions> for DirectionPaths {
	fn from(directions: Directions) -> Self {
		Self { directions }
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Directions {
	pub forward: Path,
	pub backward: Path,
	pub left: Path,
	pub right: Path,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PlayMode {
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Animation {
	pub path: AnimationAsset,
	pub play_mode: PlayMode,
}

impl Animation {
	pub fn new(path: AnimationAsset, play_mode: PlayMode) -> Self {
		Self { path, play_mode }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_movement_directions() {
		assert_eq!(
			vec![
				MovementDirection::Forward,
				MovementDirection::Backward,
				MovementDirection::Left,
				MovementDirection::Right,
			],
			MovementDirection::iterator().collect::<Vec<_>>()
		);
	}
}
