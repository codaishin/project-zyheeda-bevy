use bevy::prelude::*;

use crate::components::Animator;

pub fn add_animator<TAgent: Component>(
	mut commands: Commands,
	animators: Query<Entity, Added<AnimationPlayer>>,
	players: Query<Entity, With<TAgent>>,
	parents: Query<&Parent>,
) {
	let Ok(animator) = animators.get_single() else {
		return;
	};
	let Ok(agent) = players.get_single() else {
		return;
	};
	let animator_is_child_of_agent = parents
		.iter_ancestors(animator)
		.any(|ancestor| ancestor == agent);

	if !animator_is_child_of_agent {
		return;
	}

	commands.entity(animator).insert(Animator::<TAgent>::new());
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};

	#[derive(Component)]
	struct Agent;

	#[test]
	fn add_player_animator_to_animation_player_child_of_player() {
		let mut app = App::new();
		let player = app.world.spawn(Agent).id();
		let animator = app.world.spawn(AnimationPlayer::default()).id();
		app.world.entity_mut(player).push_children(&[animator]);
		app.update();

		app.add_systems(Update, add_animator::<Agent>);
		app.update();

		assert!(app.world.entity(animator).contains::<Animator::<Agent>>());
	}

	#[test]
	fn add_no_player_animator_to_animation_player_if_not_child_of_player() {
		let mut app = App::new();
		let animator = app.world.spawn(AnimationPlayer::default()).id();
		app.update();

		app.add_systems(Update, add_animator::<Agent>);
		app.update();

		assert!(!app.world.entity(animator).contains::<Animator::<Agent>>());
	}
}
