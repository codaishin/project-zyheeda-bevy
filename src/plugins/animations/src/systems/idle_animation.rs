use std::hash::Hash;

use super::active_animation::PlayingAnimations;
use crate::{components::Animator, resource::Animations, traits::RepeatAnimation};
use bevy::ecs::{
	component::Component,
	entity::Entity,
	query::With,
	system::{In, Query, Res},
};

pub(crate) fn idle_animation<
	TAgent: Component,
	TAnimationKey: Copy + Eq + Hash + Default + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation,
>(
	playing_animations: In<PlayingAnimations>,
	animations: Res<Animations<TAnimationKey>>,
	agents: Query<(Entity, &Animator), With<TAgent>>,
	mut animation_players: Query<&mut TAnimationPlayer>,
) {
	let key = TAnimationKey::default();
	let not_already_playing =
		|(agent, ..): &(Entity, &Animator)| !playing_animations.0.contains(agent);

	for (.., animator) in agents.iter().filter(not_already_playing) {
		play_idle_animation(&mut animation_players, &animations, animator, key);
	}
}

fn play_idle_animation<
	TAnimationKey: Copy + Eq + Hash + Default + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation,
>(
	animation_players: &mut Query<&mut TAnimationPlayer>,
	animations: &Res<Animations<TAnimationKey>>,
	animator: &Animator,
	key: TAnimationKey,
) {
	let Some(player_id) = animator.animation_player_id else {
		return;
	};
	let Ok(mut player) = animation_players.get_mut(player_id) else {
		return;
	};
	let Some(clip) = animations.0.get(&key) else {
		return;
	};
	player.repeat(clip);
}

#[cfg(test)]
mod tests {
	use crate::{components::Animator, resource::Animations, traits::RepeatAnimation};

	use super::*;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::{
			component::Component,
			entity::Entity,
			system::{IntoSystem, Res, Resource},
		},
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{automock, predicate::eq};
	use std::collections::{HashMap, HashSet};

	#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
	struct _Key;

	#[derive(Component, Default)]
	struct _AnimationPlayer {
		pub mock: Mock_AnimationPlayer,
	}

	#[derive(Component)]
	struct _Agent;

	#[automock]
	impl RepeatAnimation for _AnimationPlayer {
		fn repeat(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.repeat(animation)
		}
	}

	#[derive(Resource, Default)]
	struct _FakeAlreadyPlaying(HashSet<Entity>);

	impl _FakeAlreadyPlaying {
		fn system(fakes: Res<_FakeAlreadyPlaying>) -> PlayingAnimations {
			fakes.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.init_resource::<_FakeAlreadyPlaying>();
		app.add_systems(
			Update,
			_FakeAlreadyPlaying::system.pipe(idle_animation::<_Agent, _Key, _AnimationPlayer>),
		);

		app
	}

	#[test]
	fn play_default_key() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(Animations(HashMap::from([(_Key, handle.clone())])));

		mock_animation_player
			.mock
			.expect_repeat()
			.times(1)
			.with(eq(handle.clone()))
			.return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn((
			_Agent,
			Animator {
				animation_player_id,
			},
		));

		app.update();
	}

	#[test]
	fn do_not_play_if_no_agent_attached() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(Animations(HashMap::from([(_Key, Handle::default())])));

		mock_animation_player
			.mock
			.expect_repeat()
			.never()
			.return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn(Animator {
			animation_player_id,
		});

		app.update();
	}

	#[test]
	fn do_not_play_if_already_playing_another() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(Animations(HashMap::from([(_Key, Handle::default())])));

		mock_animation_player
			.mock
			.expect_repeat()
			.never()
			.return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		let agent = app
			.world
			.spawn((
				_Agent,
				Animator {
					animation_player_id,
				},
			))
			.id();
		app.world
			.resource_mut::<_FakeAlreadyPlaying>()
			.0
			.insert(agent);

		app.update();
	}
}
