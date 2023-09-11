#[cfg(test)]
mod move_player_tests;

use crate::{
	components::Player,
	traits::{
		movement::{Movement, Seconds},
		world_position::GetWorldPosition,
	},
};
use bevy::prelude::*;

#[inline]
fn execute_move(
	transform: &mut Transform,
	movement: &dyn Movement,
	target: Option<Vec3>,
	delta_time: Seconds,
) {
	let Some(target) = target else {
		return;
	};
	if target == transform.translation {
		return;
	}

	movement.move_towards(transform, target, delta_time);
}

pub fn move_player<
	TWorldPositionEvent: GetWorldPosition + Event,
	TMovementComponent: Movement + Component,
>(
	time: Res<Time>,
	mut event_reader: EventReader<TWorldPositionEvent>,
	mut query: Query<(&mut Player, &TMovementComponent, &mut Transform)>,
) {
	for (mut player, movement, mut transform) in query.iter_mut() {
		player.move_target = event_reader
			.iter()
			.fold(player.move_target, |_, e| e.get_world_position());
		execute_move(
			&mut transform,
			movement,
			player.move_target,
			time.delta_seconds(),
		);
	}
}
