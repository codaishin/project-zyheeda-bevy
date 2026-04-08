use bevy::prelude::*;
use common::traits::{
	handles_animations::{
		AffectedAnimationBones,
		AnimationKey,
		AnimationMaskBits,
		ForwardPitch,
		PlayMode,
	},
	iterate::Iterate,
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AnimationLookup<TAnimationClips = AnimationClips> {
	pub(crate) animations: HashMap<AnimationKey, AnimationLookupData<TAnimationClips>>,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
}

impl<TAnimationClips> Default for AnimationLookup<TAnimationClips> {
	fn default() -> Self {
		Self {
			animations: HashMap::default(),
			animation_mask_groups: HashMap::default(),
		}
	}
}

#[derive(Debug, PartialEq, Default)]
pub(crate) struct AnimationLookupData<TAnimations = AnimationClips> {
	pub(crate) animation_clips: TAnimations,
	pub(crate) play_mode: PlayMode,
	pub(crate) mask: AnimationMaskBits,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum AnimationClips {
	Single(AnimationNodeIndex),
	Directional(DirectionalIndices),
	PitchedForward(PitchedForwardIndices),
}

impl Default for AnimationClips {
	fn default() -> Self {
		Self::Single(AnimationNodeIndex::default())
	}
}

impl<'a> Iterate<'a> for AnimationClips {
	type TItem = &'a AnimationNodeIndex;
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter::from(self)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) struct DirectionalIndices {
	pub(crate) forward: AnimationNodeIndex,
	pub(crate) backward: AnimationNodeIndex,
	pub(crate) left: AnimationNodeIndex,
	pub(crate) right: AnimationNodeIndex,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) struct PitchedForwardIndices {
	pub(crate) neutral: AnimationNodeIndex,
	pub(crate) up: (ForwardPitch, AnimationNodeIndex),
	pub(crate) down: (ForwardPitch, AnimationNodeIndex),
}

pub struct Iter<'a> {
	animations: &'a AnimationClips,
	index: usize,
}

impl<'a> From<&'a AnimationClips> for Iter<'a> {
	fn from(animations: &'a AnimationClips) -> Self {
		Self {
			animations,
			index: 0,
		}
	}
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a AnimationNodeIndex;

	fn next(&mut self) -> Option<Self::Item> {
		use AnimationClips::{Directional, PitchedForward, Single};
		let index = self.index;

		self.index += 1;

		match (index, self.animations) {
			(0, Single(node_index)) => Some(node_index),
			(0, Directional(DirectionalIndices { forward, .. })) => Some(forward),
			(1, Directional(DirectionalIndices { backward, .. })) => Some(backward),
			(2, Directional(DirectionalIndices { left, .. })) => Some(left),
			(3, Directional(DirectionalIndices { right, .. })) => Some(right),
			(0, PitchedForward(PitchedForwardIndices { neutral, .. })) => Some(neutral),
			(1, PitchedForward(PitchedForwardIndices { up, .. })) => Some(&up.1),
			(2, PitchedForward(PitchedForwardIndices { down, .. })) => Some(&down.1),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_single_animation() {
		let animations = AnimationClips::Single(AnimationNodeIndex::new(42));

		assert_eq!(
			vec![&AnimationNodeIndex::new(42)],
			animations.iterate().take(2).collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_directional_animations() {
		let animations = AnimationClips::Directional(DirectionalIndices {
			forward: AnimationNodeIndex::new(11),
			backward: AnimationNodeIndex::new(20),
			left: AnimationNodeIndex::new(9),
			right: AnimationNodeIndex::new(555),
		});

		assert_eq!(
			vec![
				&AnimationNodeIndex::new(11),
				&AnimationNodeIndex::new(20),
				&AnimationNodeIndex::new(9),
				&AnimationNodeIndex::new(555),
			],
			animations.iterate().take(5).collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_pitch_animations() {
		let animations = AnimationClips::PitchedForward(PitchedForwardIndices {
			neutral: AnimationNodeIndex::new(11),
			up: (ForwardPitch::MAX, AnimationNodeIndex::new(42)),
			down: (ForwardPitch::MAX, AnimationNodeIndex::new(100)),
		});

		assert_eq!(
			vec![
				&AnimationNodeIndex::new(11),
				&AnimationNodeIndex::new(42),
				&AnimationNodeIndex::new(100),
			],
			animations.iterate().take(5).collect::<Vec<_>>()
		)
	}
}
