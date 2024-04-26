use crate::{
	components::Animator,
	resource::AnimationClips,
	traits::{RepeatAnimation, ReplayAnimation},
};
use bevy::{
	animation::AnimationClip,
	asset::Handle,
	ecs::{
		component::Component,
		entity::Entity,
		query::With,
		system::{Commands, In, Query, Res},
		world::Mut,
	},
};
use common::{components::Animate, traits::try_remove_from::TryRemoveFrom};
use std::{collections::HashSet, hash::Hash};

pub(crate) type PlayingAnimations = HashSet<Entity>;

pub(crate) fn active_animation<
	TAgent: Component,
	TAnimationKey: Clone + Copy + Hash + Eq + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	playing_animations: In<PlayingAnimations>,
	mut commands: Commands,
	animations: Res<AnimationClips<TAnimationKey>>,
	agents: Query<(Entity, &Animator, &Animate<TAnimationKey>), With<TAgent>>,
	mut animation_players: Query<&mut TAnimationPlayer>,
) -> PlayingAnimations {
	let not_already_playing = |(agent, ..): &(Entity, &Animator, &Animate<TAnimationKey>)| {
		!playing_animations.0.contains(agent)
	};
	let execute_animation = |(agent, animator, animate)| {
		play_animation(&mut animation_players, animator, animate, &animations);
		commands.try_remove_from::<Animate<TAnimationKey>>(agent);
		agent
	};
	let mut busy = agents
		.iter()
		.filter(not_already_playing)
		.filter(not_animate_none)
		.map(execute_animation)
		.collect::<HashSet<_>>();

	busy.extend(playing_animations.0);
	busy
}

fn not_animate_none<TAnimationKey: Clone + Copy + Hash + Eq + Sync + Send + 'static>(
	(.., animate): &(Entity, &Animator, &Animate<TAnimationKey>),
) -> bool {
	animate != &&Animate::<TAnimationKey>::None
}

fn play_animation<
	TAnimationKey: Clone + Copy + Hash + Eq + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	animation_players: &mut Query<&mut TAnimationPlayer>,
	animator: &Animator,
	animate: &Animate<TAnimationKey>,
	animations: &AnimationClips<TAnimationKey>,
) {
	let Some(animation_player) = get_animation_player(animator, animation_players) else {
		return;
	};

	match animate {
		Animate::Replay(key) => replay(animation_player, animations.0.get(key)),
		Animate::Repeat(key) => repeat(animation_player, animations.0.get(key)),
		_ => (),
	};
}

fn get_animation_player<'a, TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation>(
	animator: &Animator,
	animation_players: &'a mut Query<&mut TAnimationPlayer>,
) -> Option<Mut<'a, TAnimationPlayer>> {
	animator
		.animation_player_id
		.and_then(|id| animation_players.get_mut(id).ok())
}

fn replay<TAnimationPlayer: ReplayAnimation>(
	mut animation_player: Mut<TAnimationPlayer>,
	clip: Option<&Handle<AnimationClip>>,
) {
	let Some(clip) = clip else {
		return;
	};
	animation_player.replay(clip);
}

fn repeat<TAnimationPlayer: RepeatAnimation>(
	mut animation_player: Mut<TAnimationPlayer>,
	clip: Option<&Handle<AnimationClip>>,
) {
	let Some(clip) = clip else {
		return;
	};
	animation_player.repeat(clip);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, Handle},
		ecs::system::{In, IntoSystem, Resource},
		utils::Uuid,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
	enum _Key {
		A,
		B,
	}

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Default)]
	struct _AnimationPlayer {
		pub mock: Mock_AnimationPlayer,
	}

	impl RepeatAnimation for _AnimationPlayer {
		fn repeat(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.repeat(animation)
		}
	}

	impl ReplayAnimation for _AnimationPlayer {
		fn replay(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.replay(animation)
		}
	}

	mock! {
		_AnimationPlayer {}
		impl RepeatAnimation for _AnimationPlayer {
			fn repeat(&mut self, _animation: &Handle<AnimationClip>) {}
		}
		impl ReplayAnimation for _AnimationPlayer {
			fn replay(&mut self, _animation: &Handle<AnimationClip>) {}
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Playing;

	#[derive(Resource, Default)]
	struct _FakeAlreadyPlaying(HashSet<Entity>);

	impl _FakeAlreadyPlaying {
		fn system(fakes: Res<_FakeAlreadyPlaying>) -> PlayingAnimations {
			fakes.0.clone()
		}
	}

	fn track_playing(playing: In<PlayingAnimations>, mut commands: Commands) {
		for entity in playing.0 {
			commands.entity(entity).insert(_Playing);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_FakeAlreadyPlaying>();
		app.add_systems(
			Update,
			_FakeAlreadyPlaying::system
				.pipe(active_animation::<_Agent, _Key, _AnimationPlayer>)
				.pipe(track_playing),
		);

		app
	}

	#[test]
	fn replay_animation() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player
			.mock
			.expect_replay()
			.times(1)
			.with(eq(handle.clone()))
			.return_const(());
		mock_animation_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn((
			_Agent,
			Animator {
				animation_player_id,
			},
			Animate::Replay(_Key::A),
		));

		app.update();
	}

	#[test]
	fn repeat_animation() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player
			.mock
			.expect_repeat()
			.times(1)
			.with(eq(handle.clone()))
			.return_const(());
		mock_animation_player.mock.expect_replay().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn((
			_Agent,
			Animator {
				animation_player_id,
			},
			Animate::Repeat(_Key::A),
		));

		app.update();
	}

	#[test]
	fn do_not_replay_animation_when_not_with_agent() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player
			.mock
			.expect_replay()
			.never()
			.return_const(());
		mock_animation_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn((
			Animator {
				animation_player_id,
			},
			Animate::Replay(_Key::A),
		));

		app.update();
	}

	#[test]
	fn do_not_repeat_animation_when_not_with_agent() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player.mock.expect_replay().return_const(());
		mock_animation_player
			.mock
			.expect_repeat()
			.never()
			.return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		app.world.spawn((
			Animator {
				animation_player_id,
			},
			Animate::Repeat(_Key::A),
		));

		app.update();
	}

	#[test]
	fn remove_animate() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player.mock.expect_replay().return_const(());
		mock_animation_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		let agent = app
			.world
			.spawn((
				_Agent,
				Animator {
					animation_player_id,
				},
				Animate::Replay(_Key::A),
			))
			.id();

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_Key>>());
	}

	#[test]
	fn return_playing_animations() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player.mock.expect_replay().return_const(());
		mock_animation_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		let animate = app
			.world
			.spawn((
				_Agent,
				Animator {
					animation_player_id,
				},
				Animate::Replay(_Key::A),
			))
			.id();

		app.update();

		let animate = app.world.entity(animate);

		assert_eq!(Some(&_Playing), animate.get::<_Playing>());
	}

	#[test]
	fn return_incoming_playing_animations() {
		let mut app = setup();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		let fake_busy = app.world.spawn_empty().id();
		app.world
			.resource_mut::<_FakeAlreadyPlaying>()
			.0
			.insert(fake_busy);

		app.update();

		let fake_busy = app.world.entity(fake_busy);

		assert_eq!(Some(&_Playing), fake_busy.get::<_Playing>());
	}

	#[test]
	fn do_not_play_animation_when_contained_in_incoming_playing_animations() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player
			.mock
			.expect_replay()
			.never()
			.return_const(());
		mock_animation_player
			.mock
			.expect_repeat()
			.never()
			.return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		let animate = app
			.world
			.spawn((
				_Agent,
				Animator {
					animation_player_id,
				},
				Animate::Replay(_Key::A),
			))
			.id();
		app.world
			.resource_mut::<_FakeAlreadyPlaying>()
			.0
			.insert(animate);

		app.update();

		let animate = app.world.entity(animate);

		assert_eq!(Some(&_Playing), animate.get::<_Playing>());
	}
	#[test]
	fn do_not_return_playing_animations_when_animation_is_none() {
		let mut app = setup();
		let mut mock_animation_player = _AnimationPlayer::default();
		app.insert_resource(AnimationClips(HashMap::from([
			(_Key::A, Handle::default()),
			(_Key::B, Handle::default()),
		])));

		mock_animation_player.mock.expect_replay().return_const(());
		mock_animation_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_animation_player).id());
		let animate = app
			.world
			.spawn((
				_Agent,
				Animator {
					animation_player_id,
				},
				Animate::<_Key>::None,
			))
			.id();

		app.update();

		let animate = app.world.entity(animate);

		assert_eq!(None, animate.get::<_Playing>());
	}
}
