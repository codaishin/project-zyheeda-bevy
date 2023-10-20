use crate::{components::Animator, resources::Animation};
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub fn animate<TAgent: Component, TBehavior: Component>(
	animation: Res<Animation<TAgent, TBehavior>>,
	mut animators: Query<&Animator, (With<TAgent>, With<TBehavior>)>,
	mut animation_players: Query<&mut AnimationPlayer>,
) {
	for animation_player_id in animators.iter_mut().filter_map(|a| a.animation_player_id) {
		match animation_players.get_mut(animation_player_id) {
			Ok(mut animation_player) => animation_player.play(animation.clip.clone_weak()).repeat(),
			Err(e) => panic!("{}", e), //FIXME: should never happen, how to better deal with this?
		};
	}
}
