use crate::components::animation_dispatch::AnimationDispatch;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{handles_animations::AnimationKey, handles_custom_assets::TryLoadFrom},
};
use serde::{Deserialize, Serialize};
use zyheeda_core::prelude::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationDispatchDto {
	priorities: (
		OrderedSet<AnimationKey>,
		OrderedSet<AnimationKey>,
		OrderedSet<AnimationKey>,
	),
}

impl From<AnimationDispatch> for AnimationDispatchDto {
	fn from(AnimationDispatch { priorities, .. }: AnimationDispatch) -> Self {
		Self { priorities }
	}
}

impl TryLoadFrom<AnimationDispatchDto> for AnimationDispatch {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AnimationDispatchDto { priorities: stack }: AnimationDispatchDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			priorities: stack,
			..default()
		})
	}
}
