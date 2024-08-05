use crate::{
	animation::PlayMode,
	components::Animator,
	traits::{
		AnimationPath,
		AnimationPlayMode,
		HighestPriorityAnimation,
		IsPlaying,
		RepeatAnimation,
		ReplayAnimation,
	},
	Res,
};
use bevy::prelude::{Changed, Component, Query, With};
use common::{resources::Shared, traits::load_asset::Path};

type AgentQuery<'a, 'b, 'c, 'd, TAgent, TDispatch> =
	Query<'a, 'b, (&'c Animator, &'d TDispatch), (With<TAgent>, Changed<TDispatch>)>;

pub(crate) fn play_animation_clip<TAgent, TAnimation, TDispatch, TIndex, TPlayer, TTransitions>(
	agents: AgentQuery<TAgent, TDispatch>,
	mut players: Query<(&mut TPlayer, &mut TTransitions)>,
	animations: Res<Shared<Path, TIndex>>,
) where
	TAgent: Component,
	TAnimation: AnimationPlayMode + AnimationPath,
	TDispatch: Component + HighestPriorityAnimation<TAnimation>,
	TIndex: Clone + Sync + Send + 'static,
	TPlayer: Component,
	TTransitions: Component,
	for<'a> (&'a mut TPlayer, &'a mut TTransitions):
		ReplayAnimation<TIndex> + RepeatAnimation<TIndex> + IsPlaying<TIndex>,
{
	for (animator, dispatch) in &agents {
		let Some(animation) = dispatch.highest_priority_animation() else {
			continue;
		};
		let Some(index) = animations.get(animation.animation_path()) else {
			continue;
		};
		let Ok((mut player, mut transition)) = players.get_mut(animator.animation_player) else {
			continue;
		};
		let mut player = (player.as_mut(), transition.as_mut());

		if player.is_playing(index.clone()) {
			continue;
		}

		match animation.animation_play_mode() {
			PlayMode::Replay => player.replay(index.clone()),
			PlayMode::Repeat => player.repeat(index.clone()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{animation::PlayMode, components::Animator};
	use bevy::{
		app::{App, Update},
		prelude::Component,
	};
	use common::{
		resources::Shared,
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{mock, predicate::eq};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Animation {
		play_mode: PlayMode,
		path: Path,
	}

	impl AnimationPlayMode for _Animation {
		fn animation_play_mode(&self) -> PlayMode {
			self.play_mode
		}
	}

	impl AnimationPath for _Animation {
		fn animation_path(&self) -> &Path {
			&self.path
		}
	}

	#[derive(Component, Default)]
	struct _Dispatch(Option<_Animation>);

	impl HighestPriorityAnimation<_Animation> for _Dispatch {
		fn highest_priority_animation(&self) -> Option<&_Animation> {
			self.0.as_ref()
		}
	}

	#[derive(Component, Clone, Debug, PartialEq)]
	struct _Index(&'static str);

	#[derive(Component, Debug, PartialEq)]
	struct _Player;

	#[derive(Component, NestedMock)]
	struct _Transitions {
		mock: Mock_Transitions,
	}

	mock! {
		_Transitions {}
		impl IsPlaying<_Index> for _Transitions{
			fn is_playing(&self, index: _Index) -> bool;
		}
		impl ReplayAnimation<_Index> for _Transitions {
			fn replay(&mut self, index: _Index);
		}
		impl RepeatAnimation<_Index> for _Transitions {
			fn repeat(&mut self, index: _Index);
		}
	}

	impl<'a> IsPlaying<_Index> for (&'a mut _Player, &'a mut _Transitions) {
		fn is_playing(&self, index: _Index) -> bool {
			self.1.mock.is_playing(index)
		}
	}

	impl<'a> ReplayAnimation<_Index> for (&'a mut _Player, &'a mut _Transitions) {
		fn replay(&mut self, index: _Index) {
			self.1.mock.replay(index)
		}
	}

	impl<'a> RepeatAnimation<_Index> for (&'a mut _Player, &'a mut _Transitions) {
		fn repeat(&mut self, index: _Index) {
			self.1.mock.repeat(index)
		}
	}

	fn setup<const N: usize>(animations: [(Path, _Index); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(Shared::new(animations));
		app.add_systems(
			Update,
			play_animation_clip::<_Agent, _Animation, _Dispatch, _Index, _Player, _Transitions>,
		);

		app
	}

	#[test]
	fn replay_clip() {
		let mut app = setup([(Path::from("my/path"), _Index("my/path"))]);
		let animation_player = app
			.world_mut()
			.spawn((
				_Player,
				_Transitions::new_mock(|mock| {
					mock.expect_is_playing().return_const(false);
					mock.expect_repeat().never().return_const(());
					mock.expect_replay()
						.times(1)
						.with(eq(_Index("my/path")))
						.return_const(());
				}),
			))
			.id();
		app.world_mut().spawn((
			_Agent,
			Animator { animation_player },
			_Dispatch(Some(_Animation {
				play_mode: PlayMode::Replay,
				path: Path::from("my/path"),
			})),
		));

		app.update();
	}

	#[test]
	fn repeat_clip() {
		let mut app = setup([(Path::from("my/path"), _Index("my/path"))]);
		let animation_player = app
			.world_mut()
			.spawn((
				_Player,
				_Transitions::new_mock(|mock| {
					mock.expect_is_playing().return_const(false);
					mock.expect_replay().never().return_const(());
					mock.expect_repeat()
						.times(1)
						.with(eq(_Index("my/path")))
						.return_const(());
				}),
			))
			.id();
		app.world_mut().spawn((
			_Agent,
			Animator { animation_player },
			_Dispatch(Some(_Animation {
				play_mode: PlayMode::Repeat,
				path: Path::from("my/path"),
			})),
		));

		app.update();
	}

	#[test]
	fn do_not_play_when_already_playing() {
		let mut app = setup([(Path::from("my/path"), _Index("my/path"))]);
		let animation_player = app
			.world_mut()
			.spawn((
				_Player,
				_Transitions::new_mock(|mock| {
					mock.expect_is_playing()
						.with(eq(_Index("my/path")))
						.return_const(true);
					mock.expect_is_playing().return_const(false);
					mock.expect_replay().never().return_const(());
					mock.expect_repeat().never().return_const(());
				}),
			))
			.id();
		app.world_mut().spawn((
			_Agent,
			Animator { animation_player },
			_Dispatch(Some(_Animation {
				play_mode: PlayMode::Repeat,
				path: Path::from("my/path"),
			})),
		));

		app.update();
	}

	#[test]
	fn only_play_when_dispatch_changed() {
		let mut app = setup([(Path::from("my/path"), _Index("my/path"))]);
		let animation_player = app
			.world_mut()
			.spawn((
				_Player,
				_Transitions::new_mock(|mock| {
					mock.expect_is_playing().return_const(false);
					mock.expect_replay().times(1).return_const(());
					mock.expect_repeat().times(1).return_const(());
				}),
			))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				Animator { animation_player },
				_Dispatch(Some(_Animation {
					play_mode: PlayMode::Replay,
					path: Path::from("my/path"),
				})),
			))
			.id();

		app.update();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Dispatch>()
			.unwrap()
			.0 = Some(_Animation {
			play_mode: PlayMode::Repeat,
			path: Path::from("my/path"),
		});

		app.update();
	}

	#[test]
	fn do_nothing_when_agent_component_missing() {
		let mut app = setup([(Path::from("my/path"), _Index("my/path"))]);
		let animation_player = app
			.world_mut()
			.spawn((
				_Player,
				_Transitions::new_mock(|mock| {
					mock.expect_is_playing().never().return_const(false);
					mock.expect_replay().never().return_const(());
					mock.expect_repeat().never().return_const(());
				}),
			))
			.id();
		app.world_mut().spawn((
			Animator { animation_player },
			_Dispatch(Some(_Animation {
				play_mode: PlayMode::Repeat,
				path: Path::from("my/path"),
			})),
		));

		app.update();
	}
}
