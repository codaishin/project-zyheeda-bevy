use crate::components::{
	animation_dispatch::{AnimationDispatch, AnimationPlayers, AnimationState},
	animation_lookup::AnimationLookup,
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use zyheeda_core::collections::iterate::Iterate;

impl AnimationDispatch {
	pub(crate) fn write_animation_seek_state(
		dispatchers: Query<(&mut AnimationDispatch, &AnimationPlayers, &AnimationLookup)>,
		players: Query<&AnimationPlayer>,
	) {
		Self::write_animation_seek_state_internal(dispatchers, players);
	}

	fn write_animation_seek_state_internal<TClips>(
		dispatchers: Query<(
			&mut AnimationDispatch,
			&AnimationPlayers,
			&AnimationLookup<TClips>,
		)>,
		players: Query<&AnimationPlayer>,
	) where
		TClips: ThreadSafe + for<'a> Iterate<'a, TItem = &'a AnimationNodeIndex>,
	{
		for (mut dispatch, pls, lookup) in dispatchers {
			dispatch.states.clear();

			for player in pls.iter() {
				let Ok(player) = players.get(player) else {
					continue;
				};

				for (key, animation) in lookup.animations.iter() {
					let Some(id) = animation.clips.iterate().next() else {
						continue;
					};

					let Some(animation) = player.animation(*id) else {
						continue;
					};

					let seek = animation.seek_time();
					if seek.is_nan() {
						continue;
					}

					dispatch.states.insert(*key, AnimationState { seek });
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		animation_dispatch::{AnimationPlayerOf, AnimationState},
		animation_lookup::AnimationLookup,
	};
	use common::traits::handles_animations::{Animation, AnimationKey};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Default)]
	struct _Clips(Vec<AnimationNodeIndex>);

	impl<'a> Iterate<'a> for _Clips {
		type TItem = &'a AnimationNodeIndex;
		type TIter = std::slice::Iter<'a, AnimationNodeIndex>;

		fn iterate(&'a self) -> Self::TIter {
			self.0.iter()
		}
	}

	fn player_with_clips<const N: usize>(
		clips: [(AnimationNodeIndex, AnimationState); N],
	) -> AnimationPlayer {
		let mut player = AnimationPlayer::default();

		for (clip, AnimationState { seek }) in clips {
			let clip = player.play(clip);
			clip.set_seek_time(seek);
		}

		player
	}

	fn lookup_with_clips<const N: usize>(
		clips: [(AnimationKey, Vec<AnimationNodeIndex>); N],
	) -> AnimationLookup<_Clips> {
		AnimationLookup {
			animations: clips
				.into_iter()
				.map(|(key, clips)| {
					(
						key,
						Animation {
							clips: _Clips(clips),
							..default()
						},
					)
				})
				.collect(),
			..default()
		}
	}

	fn dispatch_with_states<const N: usize>(
		states: [(AnimationKey, AnimationState); N],
	) -> AnimationDispatch {
		let mut dispatch = AnimationDispatch::default();
		dispatch.states = HashMap::from(states);

		dispatch
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			AnimationDispatch::write_animation_seek_state_internal::<_Clips>,
		);

		app
	}

	#[test]
	fn write_active_animations_state() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				dispatch_with_states([]),
				lookup_with_clips([(AnimationKey::Walk, vec![AnimationNodeIndex::new(11)])]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			player_with_clips([(AnimationNodeIndex::new(11), AnimationState { seek: 11. })]),
		));

		app.update();

		assert_eq!(
			Some(&dispatch_with_states([(
				AnimationKey::Walk,
				AnimationState { seek: 11. }
			)])),
			app.world().entity(entity).get::<AnimationDispatch>(),
		);
	}

	#[test]
	fn overwrite_active_animations_state() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				dispatch_with_states([(AnimationKey::Run, AnimationState { seek: 44. })]),
				lookup_with_clips([(AnimationKey::Walk, vec![AnimationNodeIndex::new(11)])]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			player_with_clips([(AnimationNodeIndex::new(11), AnimationState { seek: 11. })]),
		));

		app.update();

		assert_eq!(
			Some(&dispatch_with_states([(
				AnimationKey::Walk,
				AnimationState { seek: 11. }
			)])),
			app.world().entity(entity).get::<AnimationDispatch>(),
		);
	}

	#[test]
	fn ignore_nan_values() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				dispatch_with_states([]),
				lookup_with_clips([(AnimationKey::Walk, vec![AnimationNodeIndex::new(11)])]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			player_with_clips([(
				AnimationNodeIndex::new(11),
				AnimationState { seek: f32::NAN },
			)]),
		));

		app.update();

		assert_eq!(
			Some(&dispatch_with_states([])),
			app.world().entity(entity).get::<AnimationDispatch>(),
		);
	}

	#[test]
	fn ignore_subsequent_clips() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				dispatch_with_states([]),
				lookup_with_clips([(
					AnimationKey::Walk,
					vec![AnimationNodeIndex::new(11), AnimationNodeIndex::new(22)],
				)]),
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			player_with_clips([
				(AnimationNodeIndex::new(11), AnimationState { seek: 11. }),
				(AnimationNodeIndex::new(22), AnimationState { seek: 22. }),
			]),
		));

		app.update();

		assert_eq!(
			Some(&dispatch_with_states([(
				AnimationKey::Walk,
				AnimationState { seek: 11. }
			)])),
			app.world().entity(entity).get::<AnimationDispatch>(),
		);
	}
}
