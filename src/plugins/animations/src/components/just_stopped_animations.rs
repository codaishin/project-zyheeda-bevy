use crate::traits::OldAnimationState;
use bevy::prelude::*;
use common::traits::handles_animations::AnimationKey;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct JustStoppedAnimations(
	pub(crate) HashMap<AnimationKey, Option<OldAnimationState>>,
);

impl<T> From<T> for JustStoppedAnimations
where
	T: Into<HashMap<AnimationKey, Option<OldAnimationState>>>,
{
	fn from(just_stopped_animations: T) -> Self {
		Self(just_stopped_animations.into())
	}
}
