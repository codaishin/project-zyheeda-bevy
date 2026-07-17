use crate::components::animation_dispatch::AnimationDispatch;
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
#[require(AnimationDispatch)]
pub(crate) struct AnimationLookup<TClips = AnimationClips<AnimationNodeIndex>> {
	pub(crate) animations: HashMap<AnimationKey, Animation<TClips>>,
	pub(crate) animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
}

#[cfg(test)]
impl<TClips> AnimationLookup<TClips> {
	pub(crate) fn with_clips<const N: usize, T>(clips: [(AnimationKey, T); N]) -> Self
	where
		T: Into<TClips>,
	{
		use common::traits::handles_animations::PlayMode;

		Self {
			animations: clips
				.into_iter()
				.map(|(key, clips)| {
					(
						key,
						Animation {
							clips: clips.into(),
							play_mode: PlayMode::default(),
							mask_groups: AnimationMaskBits::default(),
						},
					)
				})
				.collect(),
			..default()
		}
	}
}

impl<TClips> Default for AnimationLookup<TClips> {
	fn default() -> Self {
		Self {
			animations: HashMap::default(),
			animation_mask_groups: HashMap::default(),
		}
	}
}
