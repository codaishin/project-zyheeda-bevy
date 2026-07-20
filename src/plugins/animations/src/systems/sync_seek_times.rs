use crate::{
	components::{
		animation_dispatch::AnimationPlayers,
		animation_lookup::AnimationLookup,
		changed_animations::ChangedAnimations,
	},
	traits::{SetTo, UpdateAnimation},
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::traits::{
	handles_animations::AnimationKey,
	thread_safe::ThreadSafe,
	wrap_handle::{GetHandle, WrapHandle},
};
use zyheeda_core::prelude::*;

impl ChangedAnimations {
	pub(crate) fn sync_seek_times(
		changed_animations: Query<
			(
				&Self,
				&AnimationGraphHandle,
				&AnimationPlayers,
				&AnimationLookup,
			),
			Changed<Self>,
		>,
		players: Query<&mut AnimationPlayer>,
		graphs: Res<Assets<AnimationGraph>>,
		animations: Res<Assets<AnimationClip>>,
	) {
		Self::sync_seek_times_internal(changed_animations, players, graphs, animations);
	}

	#[allow(clippy::type_complexity)]
	fn sync_seek_times_internal<TClips, TPlayer, TGraph>(
		changed_animations: Query<
			(
				&Self,
				&TGraph::TComponent,
				&AnimationPlayers,
				&AnimationLookup<TClips>,
			),
			Changed<Self>,
		>,
		mut players: Query<&mut TPlayer>,
		graphs: Res<Assets<TGraph>>,
		clips: Res<Assets<AnimationClip>>,
	) where
		TClips: ThreadSafe + for<'a> Iterate<'a, TItem = &'a AnimationNodeIndex>,
		TPlayer: Component<Mutability = Mutable> + UpdateAnimation<AnimationNodeIndex>,
		TGraph: Asset + WrapHandle + GetClipId,
	{
		for (changed, graph, pls, lookup) in changed_animations {
			let Some(graph) = graphs.get(graph.get_handle()) else {
				continue;
			};

			for started in &changed.just_started {
				let Some(stopped) = get_opposite(*started) else {
					continue;
				};
				let Some(Some(stopped_state)) = changed.just_stopped.get(&stopped) else {
					continue;
				};

				let Some(started_duration) = lookup.get_duration(started, graph, &clips) else {
					continue;
				};
				let Some(stopped_duration) = lookup.get_duration(&stopped, graph, &clips) else {
					continue;
				};

				let stopped_progress = *stopped_state.0.seek / stopped_duration;
				let started_progress = 1. - stopped_progress;
				let Ok(seek_time) = F32Finite::try_from(started_duration * started_progress) else {
					continue;
				};

				for player in pls.iter() {
					let Ok(mut player) = players.get_mut(player) else {
						continue;
					};

					for clip in lookup.iter_clips(started) {
						player.update_animation(*clip, SetTo::SeekTime(seek_time));
					}
				}
			}
		}
	}
}

impl<TClips> AnimationLookup<TClips>
where
	TClips: for<'a> Iterate<'a, TItem = &'a AnimationNodeIndex>,
{
	fn get_duration<TGraph>(
		&self,
		key: &AnimationKey,
		graph: &TGraph,
		clips: &Assets<AnimationClip>,
	) -> Option<f32>
	where
		TGraph: GetClipId,
	{
		let data = self.animations.get(key)?;

		data.clips.iterate().find_map(|clip| {
			let clip_id = graph.get_clip_id(*clip)?;
			let clip = clips.get(clip_id)?;
			Some(clip.duration())
		})
	}

	fn iter_clips(&self, key: &AnimationKey) -> impl Iterator<Item = &AnimationNodeIndex> {
		self.animations
			.get(key)
			.into_iter()
			.flat_map(|data| data.clips.iterate())
	}
}

fn get_opposite(key: AnimationKey) -> Option<AnimationKey> {
	match key {
		AnimationKey::Open => Some(AnimationKey::Close),
		AnimationKey::Close => Some(AnimationKey::Open),
		_ => None,
	}
}

trait GetClipId {
	fn get_clip_id(&self, index: AnimationNodeIndex) -> Option<AssetId<AnimationClip>>;
}

impl GetClipId for AnimationGraph {
	fn get_clip_id(&self, index: AnimationNodeIndex) -> Option<AssetId<AnimationClip>> {
		let node = self.get(index)?;
		let AnimationNodeType::Clip(ref handle) = node.node_type else {
			return None;
		};

		Some(handle.id())
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
		traits::{OldAnimationState, SetTo},
	};
	use common::traits::{
		handles_animations::{Animation, AnimationKey},
		wrap_handle::GetHandle,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::HashMap, time::Duration};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component, NestedMocks)]
	struct _AnimationPlayer {
		mock: Mock_AnimationPlayer,
	}

	#[derive(Asset, TypePath)]
	struct _Graph(HashMap<AnimationNodeIndex, AssetId<AnimationClip>>);

	impl GetClipId for _Graph {
		fn get_clip_id(&self, index: AnimationNodeIndex) -> Option<AssetId<AnimationClip>> {
			self.0.get(&index).copied()
		}
	}

	impl WrapHandle for _Graph {
		type TComponent = _GraphHandle;

		fn wrap_handle(handle: Handle<Self>) -> Self::TComponent {
			_GraphHandle(handle)
		}
	}

	#[derive(Component)]
	struct _GraphHandle(Handle<_Graph>);

	impl GetHandle for _GraphHandle {
		type TAsset = _Graph;

		fn get_handle(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	#[automock]
	impl UpdateAnimation<AnimationNodeIndex> for _AnimationPlayer {
		fn update_animation(
			&mut self,
			index: AnimationNodeIndex,
			set_to: SetTo,
		) -> Option<OldAnimationState> {
			self.mock.update_animation(index, set_to)
		}
	}

	fn clip(duration: Duration) -> AnimationClip {
		let mut clip = AnimationClip::default();

		clip.set_duration(duration.as_secs_f32());

		clip
	}

	fn setup<const C: usize, const G: usize>(
		clips: [(&Handle<AnimationClip>, AnimationClip); C],
		graphs: [(&Handle<_Graph>, _Graph); G],
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut clips_assets = Assets::default();
		let mut graph_assets = Assets::default();

		for (id, asset) in clips {
			_ = clips_assets.insert(id, asset);
		}

		for (id, asset) in graphs {
			_ = graph_assets.insert(id, asset);
		}

		app.insert_resource(clips_assets);
		app.insert_resource(graph_assets);
		app.add_systems(
			Update,
			ChangedAnimations::sync_seek_times_internal::<_Clips, _AnimationPlayer, _Graph>,
		);

		app
	}

	#[test_case(AnimationKey::Open, AnimationKey::Close; "open close")]
	#[test_case(AnimationKey::Close, AnimationKey::Open; "close open")]
	fn sync_seek_time_inversely(stopped: AnimationKey, started: AnimationKey) {
		let stopped_id = AnimationNodeIndex::new(1);
		let started_id = AnimationNodeIndex::new(2);
		let stopped_handle = new_handle();
		let started_handle = new_handle();
		let graph_handle = new_handle();
		let mut app = setup(
			[
				(&stopped_handle, clip(Duration::from_secs(16))),
				(&started_handle, clip(Duration::from_secs(8))),
			],
			[(
				&graph_handle,
				_Graph(HashMap::from([
					(stopped_id, stopped_handle.id()),
					(started_id, started_handle.id()),
				])),
			)],
		);
		let entity = app
			.world_mut()
			.spawn((
				_GraphHandle(graph_handle),
				ChangedAnimations::default()
					.with_just_stopped([(
						stopped,
						Some(OldAnimationState(AnimationState {
							seek: f32_finite!(4.),
						})),
					)])
					.with_just_started([started]),
				AnimationLookup {
					animations: HashMap::from([
						(
							stopped,
							Animation {
								clips: _Clips::from(vec![stopped_id]),
								..default()
							},
						),
						(
							started,
							Animation {
								clips: _Clips::from(vec![started_id]),
								..default()
							},
						),
					]),
					..default()
				},
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			_AnimationPlayer::new().with_mock(assert_set_seek_time(started_id, f32_finite!(6.))),
		));

		app.update();

		fn assert_set_seek_time(
			started_id: AnimationNodeIndex,
			seek_time: F32Finite,
		) -> impl FnMut(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_update_animation()
					.once()
					.with(eq(started_id), eq(SetTo::SeekTime(seek_time)))
					.return_const(None);
			}
		}
	}

	#[test]
	fn act_only_once() {
		let started = AnimationKey::Open;
		let stopped = AnimationKey::Close;
		let stopped_id = AnimationNodeIndex::new(1);
		let started_id = AnimationNodeIndex::new(2);
		let stopped_handle = new_handle();
		let started_handle = new_handle();
		let graph_handle = new_handle();
		let mut app = setup(
			[
				(&stopped_handle, clip(Duration::from_secs(16))),
				(&started_handle, clip(Duration::from_secs(8))),
			],
			[(
				&graph_handle,
				_Graph(HashMap::from([
					(stopped_id, stopped_handle.id()),
					(started_id, started_handle.id()),
				])),
			)],
		);
		let entity = app
			.world_mut()
			.spawn((
				_GraphHandle(graph_handle),
				ChangedAnimations::default()
					.with_just_stopped([(
						stopped,
						Some(OldAnimationState(AnimationState {
							seek: f32_finite!(4.),
						})),
					)])
					.with_just_started([started]),
				AnimationLookup {
					animations: HashMap::from([
						(
							stopped,
							Animation {
								clips: _Clips::from(vec![stopped_id]),
								..default()
							},
						),
						(
							started,
							Animation {
								clips: _Clips::from(vec![started_id]),
								..default()
							},
						),
					]),
					..default()
				},
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			_AnimationPlayer::new().with_mock(asset_called_once),
		));

		app.update();
		app.update();

		fn asset_called_once(mock: &mut Mock_AnimationPlayer) {
			mock.expect_update_animation().once().return_const(None);
		}
	}

	#[test]
	fn act_again_if_changed_animations_changed() {
		let started = AnimationKey::Open;
		let stopped = AnimationKey::Close;
		let stopped_id = AnimationNodeIndex::new(1);
		let started_id = AnimationNodeIndex::new(2);
		let stopped_handle = new_handle();
		let started_handle = new_handle();
		let graph_handle = new_handle();
		let mut app = setup(
			[
				(&stopped_handle, clip(Duration::from_secs(16))),
				(&started_handle, clip(Duration::from_secs(8))),
			],
			[(
				&graph_handle,
				_Graph(HashMap::from([
					(stopped_id, stopped_handle.id()),
					(started_id, started_handle.id()),
				])),
			)],
		);
		let entity = app
			.world_mut()
			.spawn((
				_GraphHandle(graph_handle),
				ChangedAnimations::default()
					.with_just_stopped([(
						stopped,
						Some(OldAnimationState(AnimationState {
							seek: f32_finite!(4.),
						})),
					)])
					.with_just_started([started]),
				AnimationLookup {
					animations: HashMap::from([
						(
							stopped,
							Animation {
								clips: _Clips::from(vec![stopped_id]),
								..default()
							},
						),
						(
							started,
							Animation {
								clips: _Clips::from(vec![started_id]),
								..default()
							},
						),
					]),
					..default()
				},
			))
			.id();
		app.world_mut().spawn((
			AnimationPlayerOf(entity),
			_AnimationPlayer::new().with_mock(asset_called_once),
		));

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<ChangedAnimations>()
			.as_deref_mut();
		app.update();

		fn asset_called_once(mock: &mut Mock_AnimationPlayer) {
			mock.expect_update_animation().times(2).return_const(None);
		}
	}
}
