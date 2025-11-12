use crate::components::animation_dispatch::AnimationDispatch;
use bevy::prelude::*;
use common::{errors::Unreachable, traits::handles_custom_assets::TryLoadFrom};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, hash::Hash};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationDispatchDto<TAnimation>
where
	TAnimation: Eq + Hash,
{
	priorities: (
		HashSet<TAnimation>,
		HashSet<TAnimation>,
		HashSet<TAnimation>,
	),
}

impl<TAnimation> From<AnimationDispatch<TAnimation>> for AnimationDispatchDto<TAnimation>
where
	TAnimation: Eq + Hash,
{
	fn from(AnimationDispatch { priorities, .. }: AnimationDispatch<TAnimation>) -> Self {
		Self { priorities }
	}
}

impl<TAnimation> TryLoadFrom<AnimationDispatchDto<TAnimation>> for AnimationDispatch<TAnimation>
where
	TAnimation: Eq + Hash,
{
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AnimationDispatchDto { priorities: stack }: AnimationDispatchDto<TAnimation>,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			priorities: stack,
			..default()
		})
	}
}
