use bevy::prelude::*;
use common::traits::{animation::AnimationPath, iterate::Iterate};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AnimationLookup<TAnimations = Animations> {
	pub(crate) animations: HashMap<AnimationPath, (TAnimations, AnimationMask)>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Animations {
	Single(AnimationNodeIndex),
	Directional(DirectionalIndices),
}

impl<'a> Iterate<'a> for Animations {
	type TItem = &'a AnimationNodeIndex;
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter::from(self)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub(crate) struct DirectionalIndices {
	pub forward: AnimationNodeIndex,
	pub backward: AnimationNodeIndex,
	pub left: AnimationNodeIndex,
	pub right: AnimationNodeIndex,
}

pub struct Iter<'a> {
	animations: &'a Animations,
	index: usize,
}

impl<'a> From<&'a Animations> for Iter<'a> {
	fn from(animations: &'a Animations) -> Self {
		Self {
			animations,
			index: 0,
		}
	}
}

impl<'a> Iterator for Iter<'a> {
	type Item = &'a AnimationNodeIndex;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;

		self.index += 1;

		match (self.animations, index) {
			(Animations::Single(node_index), 0) => Some(node_index),
			(Animations::Directional(DirectionalIndices { forward, .. }), 0) => Some(forward),
			(Animations::Directional(DirectionalIndices { backward, .. }), 1) => Some(backward),
			(Animations::Directional(DirectionalIndices { left, .. }), 2) => Some(left),
			(Animations::Directional(DirectionalIndices { right, .. }), 3) => Some(right),
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_single_animation() {
		let animations = Animations::Single(AnimationNodeIndex::new(42));

		assert_eq!(
			vec![&AnimationNodeIndex::new(42)],
			animations.iterate().take(2).collect::<Vec<_>>()
		)
	}

	#[test]
	fn iter_directional_animations() {
		let animations = Animations::Directional(DirectionalIndices {
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
}
