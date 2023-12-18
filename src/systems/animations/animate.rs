use crate::{components::Animator, resources::Animation, traits::play_animation::PlayAnimation};
use bevy::prelude::*;

pub fn animate<TAgent: Component, TMarker: Component, TPlay: PlayAnimation>(
	animation: Res<Animation<TAgent, TMarker>>,
	mut animators: Query<&Animator, (With<TAgent>, Added<TMarker>)>,
	mut animation_players: Query<&mut AnimationPlayer>,
) {
	for animation_player_id in animators.iter_mut().filter_map(|a| a.animation_player_id) {
		match animation_players.get_mut(animation_player_id) {
			Ok(mut animation_player) => TPlay::play(&mut animation_player, &animation.clip),
			Err(e) => panic!("{}", e), //FIXME: should never happen, how to better deal with this?
		};
	}
}
