use bevy::prelude::*;

use crate::components::{Player, PlayerAnimator};

pub fn add_player_animator(
	mut commands: Commands,
	animators: Query<Entity, Added<AnimationPlayer>>,
	players: Query<Entity, With<Player>>,
	parents: Query<&Parent>,
) {
	let Ok(animator) = animators.get_single() else {
		return;
	};
	let Ok(player) = players.get_single() else {
		return;
	};
	let animator_child_of_player = parents
		.iter_ancestors(animator)
		.any(|ancestor| ancestor == player);

	if !animator_child_of_player {
		return;
	}

	commands.entity(animator).insert(PlayerAnimator);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::UnitsPerSecond;
	use bevy::prelude::{App, Update};

	#[test]
	fn add_player_animator_to_animation_player_child_of_player() {
		let mut app = App::new();
		let player = app
			.world
			.spawn(Player {
				movement_speed: UnitsPerSecond::new(0.),
			})
			.id();
		let animator = app.world.spawn(AnimationPlayer::default()).id();
		app.world.entity_mut(player).push_children(&[animator]);
		app.update();

		app.add_systems(Update, add_player_animator);
		app.update();

		assert!(app.world.entity(animator).contains::<PlayerAnimator>());
	}

	#[test]
	fn add_no_player_animator_to_animation_player_if_not_child_of_player() {
		let mut app = App::new();
		let animator = app.world.spawn(AnimationPlayer::default()).id();
		app.update();

		app.add_systems(Update, add_player_animator);
		app.update();

		assert!(!app.world.entity(animator).contains::<PlayerAnimator>());
	}
}
