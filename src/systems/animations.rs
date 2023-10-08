use crate::{
	behavior::{Idle, Walk},
	components::{Player, PlayerAnimator},
	resources::PlayerAnimations,
	traits::get::Get,
};
use bevy::prelude::*;

pub trait GetClip<T> {
	fn get_clip(&self) -> Handle<AnimationClip>;
}

impl GetClip<Walk> for PlayerAnimations {
	fn get_clip(&self) -> Handle<AnimationClip> {
		self.walk.clone_weak()
	}
}

impl GetClip<Idle> for PlayerAnimations {
	fn get_clip(&self) -> Handle<AnimationClip> {
		self.idle.clone_weak()
	}
}

pub fn animate<
	TBehavior,
	TBehaviors: Component + Get<TBehavior>,
	TAnimations: Resource + GetClip<TBehavior>,
>(
	animations: Res<TAnimations>,
	mut behaviors: Query<&mut TBehaviors, With<Player>>,
	mut animators: Query<&mut AnimationPlayer, With<PlayerAnimator>>,
) {
	let Ok(mut behaviors) = behaviors.get_single_mut() else {
		return; //FIXME: handle properly
	};
	let Ok(mut animator) = animators.get_single_mut() else {
		return; //FIXME: handle properly
	};

	if behaviors.get().is_none() {
		return;
	}

	animator.play(animations.get_clip()).repeat();
}
