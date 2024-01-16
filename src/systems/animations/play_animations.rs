use crate::{
	components::{Animate, Animator},
	resources::Animations,
	traits::{
		play_animation::{RepeatAnimation, ReplayAnimation},
		recourse_key::ResourceKey,
	},
};
use bevy::{
	animation::AnimationClip,
	asset::Handle,
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, Query, Res},
		world::Mut,
	},
};
use std::hash::Hash;

pub fn play_animations<
	TAnimationKey: ResourceKey + Hash + Eq + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	mut commands: Commands,
	animations: Res<Animations<TAnimationKey>>,
	agents: Query<(Entity, &Animator, &Animate<TAnimationKey>)>,
	mut animation_players: Query<&mut TAnimationPlayer>,
) {
	for (agent, animator, animate) in &agents {
		play_animation(&mut animation_players, animator, animate, &animations);
		commands.entity(agent).remove::<Animate<TAnimationKey>>();
	}
}

fn play_animation<
	TAnimationKey: ResourceKey + Hash + Eq + Sync + Send + 'static,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	animation_players: &mut Query<&mut TAnimationPlayer>,
	animator: &Animator,
	animate: &Animate<TAnimationKey>,
	animations: &Animations<TAnimationKey>,
) {
	let Some(animation_player) = get_animation_player(animator, animation_players) else {
		return;
	};

	match animate {
		Animate::Replay(key) => replay(animation_player, animations.0.get(key)),
		Animate::Repeat(key) => repeat(animation_player, animations.0.get(key)),
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
	use crate::traits::recourse_key::Iter;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, Handle},
		utils::Uuid,
	};
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
	enum _Key {
		A,
		B,
	}

	impl ResourceKey for _Key {
		fn resource_keys() -> Iter<Self> {
			Iter(None)
		}

		fn get_next(_: &Iter<Self>) -> Option<Self> {
			None
		}

		fn get_resource_path(_: &Self) -> String {
			"does not matter".to_owned()
		}
	}

	#[derive(Component)]
	struct _Player {
		pub mock: Mock_Player,
	}

	impl _Player {
		pub fn new() -> Self {
			_Player {
				mock: Mock_Player::new(),
			}
		}
	}

	impl RepeatAnimation for _Player {
		fn repeat(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.repeat(animation)
		}
	}

	impl ReplayAnimation for _Player {
		fn replay(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.replay(animation)
		}
	}

	mock! {
		_Player {}
		impl RepeatAnimation for _Player {
			fn repeat(&mut self, _animation: &Handle<AnimationClip>) {}
		}
		impl ReplayAnimation for _Player {
			fn replay(&mut self, _animation: &Handle<AnimationClip>) {}
		}
	}

	#[test]
	fn replay_animation() {
		let mut app = App::new();
		let mut mock_player = _Player::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(Animations(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_player
			.mock
			.expect_replay()
			.times(1)
			.with(eq(handle.clone()))
			.return_const(());
		mock_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_player).id());
		app.world.spawn((
			Animator {
				animation_player_id,
			},
			Animate::Replay(_Key::A),
		));

		app.add_systems(Update, play_animations::<_Key, _Player>);
		app.update();
	}

	#[test]
	fn repeat_animation() {
		let mut app = App::new();
		let mut mock_player = _Player::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(Animations(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_player
			.mock
			.expect_repeat()
			.times(1)
			.with(eq(handle.clone()))
			.return_const(());
		mock_player.mock.expect_replay().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_player).id());
		app.world.spawn((
			Animator {
				animation_player_id,
			},
			Animate::Repeat(_Key::A),
		));

		app.add_systems(Update, play_animations::<_Key, _Player>);
		app.update();
	}

	#[test]
	fn remove_animate() {
		let mut app = App::new();
		let mut mock_player = _Player::new();
		let handle = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		app.insert_resource(Animations(HashMap::from([
			(_Key::A, handle.clone()),
			(_Key::B, Handle::default()),
		])));

		mock_player.mock.expect_replay().return_const(());
		mock_player.mock.expect_repeat().return_const(());

		let animation_player_id = Some(app.world.spawn(mock_player).id());
		let agent = app
			.world
			.spawn((
				Animator {
					animation_player_id,
				},
				Animate::Replay(_Key::A),
			))
			.id();

		app.add_systems(Update, play_animations::<_Key, _Player>);
		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Animate<_Key>>());
	}
}
