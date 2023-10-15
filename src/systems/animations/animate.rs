use crate::{components::Animator, resources::Animation, traits::get::Get};
use bevy::prelude::*;

fn active_agents<TBehavior, TBehaviors: Component + Get<TBehavior>>(
	agent: (&TBehaviors, &Animator),
) -> Option<Entity> {
	let (behaviors, animator) = agent;
	behaviors.get().and(animator.animation_player_id)
}

pub fn animate<
	TAgent: Component,
	TBehaviors: Component + Get<TBehavior>,
	TBehavior: Send + Sync + 'static,
>(
	animation: Res<Animation<TAgent, TBehavior>>,
	mut agents: Query<(&TBehaviors, &Animator), With<TAgent>>,
	mut animation_players: Query<&mut AnimationPlayer>,
) {
	for animation_player_id in agents
		.iter_mut()
		.filter_map(active_agents::<TBehavior, TBehaviors>)
	{
		match animation_players.get_mut(animation_player_id) {
			Ok(mut animation_player) => animation_player.play(animation.clip.clone_weak()).repeat(),
			Err(e) => panic!("{}", e), //FIXME: should never happen, how to better deal with this?
		};
	}
}
