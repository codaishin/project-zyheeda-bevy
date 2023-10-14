use crate::{components::Animator, resources::Animation, traits::get::Get};
use bevy::prelude::*;

pub fn animate<
	TBehavior: Send + Sync + 'static,
	TBehaviors: Component + Get<TBehavior>,
	TAgent: Component,
>(
	animation: Res<Animation<TAgent, TBehavior>>,
	mut behaviors: Query<&mut TBehaviors, With<TAgent>>,
	mut animators: Query<&mut AnimationPlayer, With<Animator<TAgent>>>,
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

	animator.play(animation.clip.clone_weak()).repeat();
}
