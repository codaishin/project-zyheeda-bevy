use crate::components::{
	animation_dispatch::{AnimationDispatch, AnimationPlayerOf},
	animation_lookup::AnimationLookup,
	changed_animations::ChangedAnimations,
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::thread_safe::ThreadSafe;
use zyheeda_core::collections::iterate::Iterate;

impl AnimationDispatch {
	pub(crate) fn apply_seek_times(
		dispatchers: Query<(&AnimationDispatch, &AnimationLookup, &mut ChangedAnimations)>,
		players: Query<(&mut AnimationPlayer, &AnimationPlayerOf), Added<AnimationPlayerOf>>,
	) {
		Self::apply_seek_times_internal(dispatchers, players);
	}

	fn apply_seek_times_internal<TClips, TPlayer>(
		mut dispatchers: Query<(
			&AnimationDispatch,
			&AnimationLookup<TClips>,
			&mut ChangedAnimations,
		)>,
		players: Query<(&mut TPlayer, &AnimationPlayerOf), Added<AnimationPlayerOf>>,
	) where
		TClips: ThreadSafe + for<'a> Iterate<'a, TItem = &'a AnimationNodeIndex>,
		TPlayer: Component<Mutability = Mutable> + SetAnimationSeekTime,
	{
		for (mut player, AnimationPlayerOf(dispatch)) in players {
			let Ok((dispatch, lookup, mut changes)) = dispatchers.get_mut(*dispatch) else {
				continue;
			};

			for (key, state) in &dispatch.states {
				if *state.seek > 0. {
					changes.just_started.remove(key);
				}

				let Some(data) = lookup.animations.get(key) else {
					continue;
				};

				for id in data.clips.iterate() {
					player.set_animation_seek_time(*id, *state.seek);
				}
			}
		}
	}
}

trait SetAnimationSeekTime {
	fn set_animation_seek_time(&mut self, id: AnimationNodeIndex, seek_time: f32);
}

impl SetAnimationSeekTime for AnimationPlayer {
	fn set_animation_seek_time(&mut self, id: AnimationNodeIndex, seek_time: f32) {
		self.animation_mut(id).map(|a| a.set_seek_time(seek_time));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			animation_dispatch::{AnimationPlayerOf, AnimationState},
			animation_lookup::AnimationLookup,
		},
		systems::write_animation_seek_state::tests::_Clips,
	};
	use common::traits::handles_animations::AnimationKey;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};
	use zyheeda_core::prelude::*;

	#[derive(Component, NestedMocks)]
	struct _AnimationPlayer {
		mock: Mock_AnimationPlayer,
	}

	#[automock]
	impl SetAnimationSeekTime for _AnimationPlayer {
		fn set_animation_seek_time(&mut self, id: AnimationNodeIndex, seek_time: f32) {
			self.mock.set_animation_seek_time(id, seek_time);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			AnimationDispatch::apply_seek_times_internal::<_Clips, _AnimationPlayer>,
		);

		app
	}

	#[test]
	fn apply_seek_time() {
		let mut app = setup();
		let dispatch = app
			.world_mut()
			.spawn((
				AnimationDispatch::with_states([(
					AnimationKey::Walk,
					AnimationState {
						seek: f32_finite!(11.),
					},
				)]),
				AnimationLookup::<_Clips>::with_clips([(
					AnimationKey::Walk,
					vec![AnimationNodeIndex::new(42)],
				)]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(dispatch),
			_AnimationPlayer::new().with_mock(assert_set_seek_time),
		));

		app.update();

		fn assert_set_seek_time(mock: &mut Mock_AnimationPlayer) {
			mock.expect_set_animation_seek_time()
				.with(eq(AnimationNodeIndex::new(42)), eq(11.))
				.return_const(());
		}
	}

	#[test]
	fn remove_from_just_started() {
		let mut app = setup();
		let dispatch = app
			.world_mut()
			.spawn((
				ChangedAnimations::default()
					.with_just_started([AnimationKey::Walk, AnimationKey::Close]),
				AnimationDispatch::with_states([(
					AnimationKey::Walk,
					AnimationState {
						seek: f32_finite!(11.),
					},
				)]),
				AnimationLookup::<_Clips>::with_clips([(
					AnimationKey::Walk,
					vec![AnimationNodeIndex::new(42)],
				)]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(dispatch),
			_AnimationPlayer::new().with_mock(|mock| {
				mock.expect_set_animation_seek_time().return_const(());
			}),
		));

		app.update();

		assert_eq!(
			Some(&ChangedAnimations::default().with_just_started([AnimationKey::Close])),
			app.world().entity(dispatch).get::<ChangedAnimations>()
		);
	}

	#[test]
	fn do_not_remove_from_just_started_if_seek_is_zero() {
		let mut app = setup();
		let dispatch = app
			.world_mut()
			.spawn((
				ChangedAnimations::default()
					.with_just_started([AnimationKey::Walk, AnimationKey::Close]),
				AnimationDispatch::with_states([(
					AnimationKey::Walk,
					AnimationState {
						seek: F32Finite::ZERO,
					},
				)]),
				AnimationLookup::<_Clips>::with_clips([(
					AnimationKey::Walk,
					vec![AnimationNodeIndex::new(42)],
				)]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(dispatch),
			_AnimationPlayer::new().with_mock(|mock| {
				mock.expect_set_animation_seek_time().return_const(());
			}),
		));

		app.update();

		assert_eq!(
			Some(
				&ChangedAnimations::default()
					.with_just_started([AnimationKey::Walk, AnimationKey::Close])
			),
			app.world().entity(dispatch).get::<ChangedAnimations>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let dispatch = app
			.world_mut()
			.spawn((
				AnimationDispatch::with_states([(
					AnimationKey::Walk,
					AnimationState {
						seek: f32_finite!(11.),
					},
				)]),
				AnimationLookup::<_Clips>::with_clips([(
					AnimationKey::Walk,
					vec![AnimationNodeIndex::new(42)],
				)]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(dispatch),
			_AnimationPlayer::new().with_mock(assert_set_seek_time),
		));

		app.update();
		app.update();

		fn assert_set_seek_time(mock: &mut Mock_AnimationPlayer) {
			mock.expect_set_animation_seek_time()
				.with(eq(AnimationNodeIndex::new(42)), eq(11.))
				.once()
				.return_const(());
		}
	}
}
