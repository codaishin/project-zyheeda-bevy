use crate::{
	animation::PlayMode,
	components::Animator,
	resource::AnimationClips,
	traits::{
		AnimationPath,
		AnimationPlayMode,
		HighestPriorityAnimation,
		RepeatAnimation,
		ReplayAnimation,
	},
};
use bevy::ecs::{
	change_detection::DetectChanges,
	component::Component,
	system::{Query, Res},
	world::Ref,
};
use common::traits::load_asset::Path;

pub(crate) fn play_animation_clip<
	TAnimation: AnimationPath + AnimationPlayMode + Sync + Send + 'static,
	TAnimationDispatch: Component + HighestPriorityAnimation<TAnimation>,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	clips: Res<AnimationClips<Path>>,
	agents: Query<(Ref<TAnimationDispatch>, Ref<Animator>)>,
	mut players: Query<&mut TAnimationPlayer>,
) {
	for (dispatch, animator) in &agents {
		play_animation(&mut players, dispatch, animator, &clips);
	}
}

fn play_animation<
	TAnimation: AnimationPath + AnimationPlayMode + Sync + Send + 'static,
	TAnimationDispatch: Component + HighestPriorityAnimation<TAnimation>,
	TAnimationPlayer: Component + RepeatAnimation + ReplayAnimation,
>(
	players: &mut Query<&mut TAnimationPlayer, ()>,
	dispatch: Ref<TAnimationDispatch>,
	animator: Ref<Animator>,
	clips: &Res<AnimationClips<Path>>,
) {
	if !dispatch.is_changed() && !animator.is_changed() {
		return;
	}
	let Some(animation) = dispatch.highest_priority_animation() else {
		return;
	};
	let Some(player_id) = animator.animation_player_id else {
		return;
	};
	let Ok(mut player) = players.get_mut(player_id) else {
		return;
	};
	let Some(clip) = clips.0.get(animation.animation_path()) else {
		return;
	};

	match animation.animation_play_mode() {
		PlayMode::Repeat => player.repeat(clip),
		PlayMode::Replay => player.replay(clip),
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::animation::PlayMode;
	use bevy::{
		animation::AnimationClip,
		app::{App, Update},
		asset::{AssetId, Handle},
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::{mock, predicate::eq};
	use std::collections::HashMap;
	use uuid::Uuid;

	struct _Animation(Path, PlayMode);

	impl AnimationPath for _Animation {
		fn animation_path(&self) -> &Path {
			&self.0
		}
	}

	impl AnimationPlayMode for _Animation {
		fn animation_play_mode(&self) -> PlayMode {
			self.1
		}
	}

	#[derive(Component)]
	struct _AnimationDispatch(Option<_Animation>);

	impl HighestPriorityAnimation<_Animation> for _AnimationDispatch {
		fn highest_priority_animation(&self) -> Option<&_Animation> {
			self.0.as_ref()
		}
	}

	#[derive(Component, Default)]
	struct _AnimationPlayer {
		mock: Mock_AnimationPlayer,
	}

	impl ReplayAnimation for _AnimationPlayer {
		fn replay(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.replay(animation)
		}
	}

	impl RepeatAnimation for _AnimationPlayer {
		fn repeat(&mut self, animation: &Handle<AnimationClip>) {
			self.mock.repeat(animation)
		}
	}

	mock! {
		_AnimationPlayer {}
		impl ReplayAnimation for _AnimationPlayer {
			fn replay(&mut self, animation: &Handle<AnimationClip>);
		}
		impl RepeatAnimation for _AnimationPlayer {
			fn repeat(&mut self, animation: &Handle<AnimationClip>);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<AnimationClips<Path>>();
		app.add_systems(
			Update,
			play_animation_clip::<_Animation, _AnimationDispatch, _AnimationPlayer>,
		);

		app
	}

	#[test]
	fn repeat_animation() {
		let mut app = setup();
		let mut player = _AnimationPlayer::default();

		let path = Path::from("a/path");
		let clip = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let dispatch = _AnimationDispatch(Some(_Animation(path.clone(), PlayMode::Repeat)));

		app.insert_resource(AnimationClips(HashMap::from([(path, clip.clone())])));

		player.mock.expect_replay().return_const(());
		player
			.mock
			.expect_repeat()
			.times(1)
			.with(eq(clip))
			.return_const(());
		let player = app.world_mut().spawn(player).id();

		app.world_mut().spawn((
			dispatch,
			Animator {
				animation_player_id: Some(player),
			},
		));
		app.update();
	}

	#[test]
	fn replay_animation() {
		let mut app = setup();
		let mut player = _AnimationPlayer::default();

		let path = Path::from("my/path");
		let clip = Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		});
		let dispatch = _AnimationDispatch(Some(_Animation(path.clone(), PlayMode::Replay)));

		app.insert_resource(AnimationClips(HashMap::from([(path, clip.clone())])));

		player.mock.expect_repeat().return_const(());
		player
			.mock
			.expect_replay()
			.times(1)
			.with(eq(clip))
			.return_const(());
		let player = app.world_mut().spawn(player).id();

		app.world_mut().spawn((
			dispatch,
			Animator {
				animation_player_id: Some(player),
			},
		));
		app.update();
	}

	#[test]
	fn only_repeat_when_dispatch_changed() {
		let mut app = setup();
		let mut player = _AnimationPlayer::default();

		let path_1 = Path::from("path/1");
		let path_2 = Path::from("path/2");
		let dispatch = _AnimationDispatch(Some(_Animation(path_1.clone(), PlayMode::Repeat)));

		app.insert_resource(AnimationClips(HashMap::from([
			(path_1, Handle::default()),
			(path_2.clone(), Handle::default()),
		])));

		player.mock.expect_replay().return_const(());
		player.mock.expect_repeat().times(2).return_const(());
		let player = app.world_mut().spawn(player).id();

		let agent = app
			.world_mut()
			.spawn((
				dispatch,
				Animator {
					animation_player_id: Some(player),
				},
			))
			.id();
		app.update();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_AnimationDispatch>()
			.unwrap()
			.0 = Some(_Animation(path_2, PlayMode::Repeat));
		app.update();
	}

	#[test]
	fn also_repeat_when_animator_changed() {
		let mut app = setup();
		let mut player = _AnimationPlayer::default();

		let path_1 = Path::from("path/a");
		let path_2 = Path::from("path/b");
		let dispatch = _AnimationDispatch(Some(_Animation(path_1.clone(), PlayMode::Repeat)));

		app.insert_resource(AnimationClips(HashMap::from([
			(path_1, Handle::default()),
			(path_2, Handle::default()),
		])));

		player.mock.expect_replay().return_const(());
		player.mock.expect_repeat().times(1).return_const(());
		let player = app.world_mut().spawn(player).id();

		let agent = app
			.world_mut()
			.spawn((
				dispatch,
				Animator {
					animation_player_id: None,
				},
			))
			.id();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<Animator>()
			.unwrap()
			.animation_player_id = Some(player);
		app.update();
	}
}
