use bevy::prelude::*;
use common::traits::handles_animations::{
	AffectedAnimationBones,
	Animation,
	AnimationClips,
	AnimationKey,
	AnimationMaskBits,
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AnimationLookup<TClips = AnimationClips<AnimationNodeIndex>> {
	pub(crate) animations: HashMap<AnimationKey, Animation<TClips>>,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
}

impl<TClips> Default for AnimationLookup<TClips> {
	fn default() -> Self {
		Self {
			animations: HashMap::default(),
			animation_mask_groups: HashMap::default(),
		}
	}
}
