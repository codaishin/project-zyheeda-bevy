use crate::components::Animator;
use bevy::prelude::*;

pub(crate) fn link_animators_with_new_animation_players(
	mut animators: Query<&mut Animator>,
	animation_players: Query<Entity, Added<AnimationPlayer>>,
	parents: Query<&Parent>,
) {
	for animation_player in &animation_players {
		link_with_parent_animators(animation_player, &parents, &mut animators);
	}
}

fn link_with_parent_animators(
	animation_player: Entity,
	parents: &Query<&Parent>,
	animators: &mut Query<&mut Animator>,
) {
	for parent in parents.iter_ancestors(animation_player) {
		link_with_parent_animator(animation_player, parent, animators)
	}
}

fn link_with_parent_animator(
	animation_player: Entity,
	parent: Entity,
	animators: &mut Query<&mut Animator>,
) {
	let Ok(mut animator) = animators.get_mut(parent) else {
		return;
	};
	animator.animation_player_id = Some(animation_player);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};

	#[test]
	fn link_animator_with_animation_player() {
		let mut app = App::new();
		let animator = app.world_mut().spawn(Animator { ..default() }).id();
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut()
			.entity_mut(animator)
			.push_children(&[animation_player]);
		app.update();

		app.add_systems(Update, link_animators_with_new_animation_players);
		app.update();

		let animator = app.world().entity(animator).get::<Animator>().unwrap();

		assert_eq!(Some(animation_player), animator.animation_player_id);
	}

	#[test]
	fn do_not_link_animator_with_animation_player_when_not_child_parent_relationship() {
		let mut app = App::new();
		let animator = app.world_mut().spawn(Animator { ..default() }).id();
		app.world_mut().spawn(AnimationPlayer::default());
		app.update();

		app.add_systems(Update, link_animators_with_new_animation_players);
		app.update();

		let animator = app.world().entity(animator).get::<Animator>().unwrap();

		assert!(animator.animation_player_id.is_none());
	}

	#[test]
	fn link_multiple_animators_with_animation_players() {
		let mut app = App::new();
		let animators = [
			app.world_mut().spawn(Animator { ..default() }).id(),
			app.world_mut().spawn(Animator { ..default() }).id(),
			app.world_mut().spawn(Animator { ..default() }).id(),
		];
		let animation_players = [
			app.world_mut().spawn(AnimationPlayer::default()).id(),
			app.world_mut().spawn(AnimationPlayer::default()).id(),
			app.world_mut().spawn(AnimationPlayer::default()).id(),
		];
		for i in 0..3 {
			app.world_mut()
				.entity_mut(animators[i])
				.push_children(&[animation_players[i]]);
		}
		app.update();

		app.add_systems(Update, link_animators_with_new_animation_players);
		app.update();

		let animation_players = animation_players.map(Some);
		let animators = animators.map(|a| {
			app.world()
				.entity(a)
				.get::<Animator>()
				.unwrap()
				.animation_player_id
		});

		assert_eq!(animation_players, animators);
	}
}
