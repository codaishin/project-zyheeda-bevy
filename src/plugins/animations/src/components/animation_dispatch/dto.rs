use crate::components::animation_dispatch::AnimationDispatch;
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{animation::Animation, handles_custom_assets::TryLoadFrom},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationDispatchDto {
	stack: (HashSet<Animation>, HashSet<Animation>, HashSet<Animation>),
}

impl From<AnimationDispatch> for AnimationDispatchDto {
	fn from(AnimationDispatch { stack, .. }: AnimationDispatch) -> Self {
		Self { stack }
	}
}

impl TryLoadFrom<AnimationDispatchDto> for AnimationDispatch {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AnimationDispatchDto { stack }: AnimationDispatchDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self { stack, ..default() })
	}
}
