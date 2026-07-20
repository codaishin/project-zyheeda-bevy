use crate::traits::OldAnimationState;
use bevy::prelude::*;
use common::traits::handles_animations::AnimationKey;
use std::collections::{HashMap, HashSet};

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ChangedAnimations {
	pub(crate) just_stopped: HashMap<AnimationKey, Option<OldAnimationState>>,
	pub(crate) just_started: HashSet<AnimationKey>,
}

#[cfg(test)]
impl ChangedAnimations {
	pub(crate) fn with_just_stopped(
		mut self,
		just_stopped: impl Into<HashMap<AnimationKey, Option<OldAnimationState>>>,
	) -> Self {
		self.just_stopped = just_stopped.into();
		self
	}

	pub(crate) fn with_just_started(
		mut self,
		just_started: impl Into<HashSet<AnimationKey>>,
	) -> Self {
		self.just_started = just_started.into();
		self
	}
}
