use crate::{
	components::{animation_dispatch::AnimationDispatch, animation_lookup::AnimationLookup},
	traits::{
		AnimationPlayers,
		GetActiveAnimations,
		IsPlaying,
		RepeatAnimation,
		ReplayAnimation,
		StopAnimation,
		asset_server::animation_graph::GetNodeMut,
	},
};
use bevy::{ecs::query::QueryData, prelude::*};
use common::traits::{
	animation::{Animation, AnimationPriority, PlayMode},
	iterate::Iterate,
	thread_safe::ThreadSafe,
	wrap_handle::{UnwrapHandle, WrapHandle},
};
use std::collections::HashSet;

const ANIMATION_PRIORITY_ORDER: [AnimationPriority; 3] = [
	AnimationPriority::High,
	AnimationPriority::Medium,
	AnimationPriority::Low,
];

impl<TDispatch> PlayAnimationClip for TDispatch {}

pub(crate) trait PlayAnimationClip
where
	Self: Sized,
{
	fn play_animation_clip_via<TAnimationPlayer>(
		players: Query<TAnimationPlayer>,
		dispatchers: Query<
			(&AnimationDispatch, &AnimationLookup, &AnimationGraphHandle),
			Changed<AnimationDispatch>,
		>,
		graphs: ResMut<Assets<AnimationGraph>>,
	) where
		Self: Component + AnimationPlayers + GetActiveAnimations<Animation>,
		TAnimationPlayer: QueryData,
		for<'a> TAnimationPlayer::Item<'a>: IsPlaying<AnimationNodeIndex>
			+ ReplayAnimation<AnimationNodeIndex>
			+ RepeatAnimation<AnimationNodeIndex>
			+ StopAnimation<AnimationNodeIndex>,
	{
		play_animation_clip_via(players, dispatchers, graphs)
	}
}

#[allow(clippy::type_complexity)]
fn play_animation_clip_via<TAnimationPlayer, TDispatch, TGraph, TAnimations>(
	mut players: Query<TAnimationPlayer>,
	agents: Query<
		(
			&TDispatch,
			&AnimationLookup<TAnimations>,
			&TGraph::TComponent,
		),
		Changed<TDispatch>,
	>,
	mut graphs: ResMut<Assets<TGraph>>,
) where
	TAnimationPlayer: QueryData,
	TGraph: Asset + GetNodeMut + WrapHandle,
	TDispatch: Component + AnimationPlayers + GetActiveAnimations<Animation>,
	for<'a> TAnimations: ThreadSafe + Iterate<'a, TItem = &'a AnimationNodeIndex>,
	for<'a> TAnimationPlayer::Item<'a>: IsPlaying<AnimationNodeIndex>
		+ ReplayAnimation<AnimationNodeIndex>
		+ RepeatAnimation<AnimationNodeIndex>
		+ StopAnimation<AnimationNodeIndex>,
{
	for (dispatcher, lookup, graph_component) in &agents {
		for entity in dispatcher.animation_players() {
			let graph_handle = graph_component.unwrap();
			let Ok(mut player) = players.get_mut(entity) else {
				continue;
			};
			let Some(graph) = graphs.get_mut(graph_handle) else {
				continue;
			};
			let active_animations = play_active(graph, &mut player, lookup, dispatcher);
			let is_inactive = |(indices, _): &(TAnimations, AnimationMask)| {
				if indices.iterate().any(|i| active_animations.contains(i)) {
					return None;
				}
				Some(indices.iterate().copied().collect::<Vec<_>>())
			};
			let active_animations = lookup.animations.values().filter_map(is_inactive).flatten();
			stop(player, graph, active_animations);
		}
	}
}

fn play_active<TPlayer, TDispatcher, TGraph, TAnimations>(
	graph: &mut TGraph,
	player: &mut TPlayer,
	lookup: &AnimationLookup<TAnimations>,
	dispatcher: &TDispatcher,
) -> HashSet<AnimationNodeIndex>
where
	TPlayer: IsPlaying<AnimationNodeIndex>
		+ ReplayAnimation<AnimationNodeIndex>
		+ RepeatAnimation<AnimationNodeIndex>,
	TDispatcher: GetActiveAnimations<Animation>,
	TGraph: Asset + GetNodeMut,
	for<'a> TAnimations: Iterate<'a, TItem = &'a AnimationNodeIndex>,
{
	let mut higher_priority_mask = 0;
	let mut active_animations = HashSet::default();

	for priority in ANIMATION_PRIORITY_ORDER {
		let blocked_by_higher_priority = higher_priority_mask;

		for active_animation in dispatcher.get_active_animations(priority) {
			let Some((ids, mask)) = lookup.animations.get(&active_animation.path) else {
				continue;
			};

			for id in ids.iterate() {
				let Some(animation_node) = graph.get_node_mut(*id) else {
					continue;
				};

				active_animations.insert(*id);
				animation_node.remove_mask(*mask);
				animation_node.add_mask(blocked_by_higher_priority);
				add(&mut higher_priority_mask, *mask);

				if player.is_playing(*id) {
					continue;
				}

				match active_animation.play_mode {
					PlayMode::Repeat => player.repeat(*id),
					PlayMode::Replay => player.replay(*id),
				}
			}
		}
	}

	active_animations
}

fn stop<TPlayer, TGraph>(
	mut player: TPlayer,
	graph: &mut TGraph,
	animations: impl Iterator<Item = AnimationNodeIndex>,
) where
	TPlayer: StopAnimation<AnimationNodeIndex>,
	TGraph: GetNodeMut,
{
	for animation_id in animations {
		if let Some(node) = graph.get_node_mut(animation_id) {
			node.add_mask(AnimationMask::MAX);
		}
		player.stop_animation(animation_id);
	}
}

fn add(dst: &mut AnimationMask, src: AnimationMask) {
	*dst |= src;
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::leak_iterator;
	use common::traits::animation::AnimationPath;
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{
		collections::{HashMap, VecDeque},
		ops::DerefMut,
		slice::Iter,
	};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl AnimationPlayers for _AnimationDispatch {
		type TIter = _Iter;

		fn animation_players(&self) -> Self::TIter {
			self.mock.animation_players()
		}
	}

	impl GetActiveAnimations<Animation> for _AnimationDispatch {
		type TIter<'a>
			= Iter<'a, Animation>
		where
			Self: 'a,
			Animation: 'a;

		fn get_active_animations<TPriority>(&self, priority: TPriority) -> Self::TIter<'_>
		where
			TPriority: Into<AnimationPriority> + 'static,
		{
			self.mock.get_active_animations(priority)
		}
	}

	mock! {
		_AnimationDispatch {}
		impl AnimationPlayers for _AnimationDispatch {
			type TIter = _Iter;

			fn animation_players(&self) -> _Iter;
		}
		impl GetActiveAnimations<Animation> for _AnimationDispatch {
			type TIter<'a>
				= Iter<'a, Animation>
			where
				Self: 'a,
				Animation: 'a;

			fn get_active_animations<TPriority>(&self, priority: TPriority) -> Iter<'static, Animation>
			where
				TPriority: Into<AnimationPriority> + 'static;
		}

	}

	#[derive(Clone)]
	struct _Animations(Vec<AnimationNodeIndex>);

	impl<'a> Iterate<'a> for _Animations {
		type TItem = &'a AnimationNodeIndex;
		type TIter = Iter<'a, AnimationNodeIndex>;

		fn iterate(&'a self) -> Iter<'a, AnimationNodeIndex> {
			self.0.iter()
		}
	}

	impl From<&[AnimationNodeIndex]> for _Animations {
		fn from(animations: &[AnimationNodeIndex]) -> Self {
			let animations = Vec::from_iter(animations.iter().copied());
			Self(animations)
		}
	}

	impl From<&Vec<AnimationNodeIndex>> for _Animations {
		fn from(animations: &Vec<AnimationNodeIndex>) -> Self {
			let animations = Vec::from_iter(animations.iter().copied());
			Self(animations)
		}
	}

	#[derive(Asset, TypePath, Default)]
	struct _Graph {
		nodes: HashMap<usize, AnimationGraphNode>,
	}

	impl WrapHandle for _Graph {
		type TComponent = _GraphComponent;

		fn wrap(handle: Handle<Self>) -> Self::TComponent {
			_GraphComponent(handle)
		}
	}

	impl GetNodeMut for _Graph {
		fn get_node_mut(
			&mut self,
			animation: AnimationNodeIndex,
		) -> Option<&'_ mut AnimationGraphNode> {
			self.nodes.get_mut(&animation.index())
		}
	}

	impl<const N: usize> From<[(usize, AnimationGraphNode); N]> for _Graph {
		fn from(nodes: [(usize, AnimationGraphNode); N]) -> Self {
			Self {
				nodes: HashMap::from(nodes),
			}
		}
	}

	#[derive(Component)]
	struct _GraphComponent(Handle<_Graph>);

	impl UnwrapHandle for _GraphComponent {
		type TAsset = _Graph;

		fn unwrap(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	#[derive(Clone)]
	struct _Iter(VecDeque<Entity>);

	impl<const N: usize> From<[Entity; N]> for _Iter {
		fn from(value: [Entity; N]) -> Self {
			_Iter(VecDeque::from(value))
		}
	}

	impl Iterator for _Iter {
		type Item = Entity;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.pop_front()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _AnimationPlayer {
		mock: Mock_AnimationPlayer,
	}

	impl Default for _AnimationPlayer {
		fn default() -> Self {
			let mut mock = Mock_AnimationPlayer::default();
			mock.expect_replay().return_const(());
			mock.expect_repeat().return_const(());
			mock.expect_stop_animation().return_const(());
			mock.expect_is_playing().return_const(false);
			Self { mock }
		}
	}

	impl IsPlaying<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn is_playing(&self, index: AnimationNodeIndex) -> bool {
			self.mock.is_playing(index)
		}
	}

	impl ReplayAnimation<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn replay(&mut self, index: AnimationNodeIndex) {
			self.mock.replay(index)
		}
	}

	impl RepeatAnimation<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn repeat(&mut self, index: AnimationNodeIndex) {
			self.mock.repeat(index)
		}
	}

	impl StopAnimation<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn stop_animation(&mut self, index: AnimationNodeIndex) {
			self.mock.stop_animation(index)
		}
	}

	mock! {
		_AnimationPlayer {}
		impl ReplayAnimation<AnimationNodeIndex> for _AnimationPlayer {
			fn replay(&mut self, index: AnimationNodeIndex);
		}
		impl RepeatAnimation<AnimationNodeIndex> for _AnimationPlayer {
			fn repeat(&mut self, index: AnimationNodeIndex);
		}
		impl StopAnimation<AnimationNodeIndex> for _AnimationPlayer{
			fn stop_animation(&mut self, index: AnimationNodeIndex);
		}
		impl IsPlaying<AnimationNodeIndex> for _AnimationPlayer {
			fn is_playing(&self, index: AnimationNodeIndex) -> bool;
		}
	}

	macro_rules! binary_str {
		($a:expr) => {{
			let values = $a
				.into_iter()
				.map(|v| format!("{v:b}"))
				.collect::<Vec<_>>()
				.join(", ");
			format!("[{values}]")
		}};
	}

	macro_rules! setup {
		($animations:expr, $graph_handle:expr) => {
			setup($animations, $graph_handle, AnimationMask::default())
		};
		($animations:expr, $graph_handle:expr, $initial_mask:expr) => {
			setup($animations, $graph_handle, $initial_mask)
		};
	}

	fn setup(
		lookup: &AnimationLookup<_Animations>,
		graph_handle: &Handle<_Graph>,
		initial_mask: AnimationMask,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();
		let mut graph = _Graph::default();

		for (animations, _) in lookup.animations.values() {
			for animation in animations.iterate() {
				graph.nodes.insert(
					animation.index(),
					AnimationGraphNode {
						mask: initial_mask,
						..default()
					},
				);
			}
		}

		graphs.insert(graph_handle, graph);
		app.insert_resource(graphs);
		app.add_systems(
			Update,
			play_animation_clip_via::<&mut _AnimationPlayer, _AnimationDispatch, _Graph, _Animations>,
		);

		app
	}

	#[test]
	fn repeat_animation() {
		let handle = new_handle();
		let indices = vec![
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&indices), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat(indices)))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path"),
						PlayMode::Repeat,
					)]));
				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();

		fn assert_repeat(indices: Vec<AnimationNodeIndex>) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_is_playing().return_const(false);
				mock.expect_replay().never().return_const(());
				for index in indices.clone() {
					mock.expect_repeat()
						.times(1)
						.with(eq(index))
						.return_const(());
				}
				mock.expect_stop_animation().never().return_const(());
			}
		}
	}

	#[test]
	fn replay_animation() {
		let handle = new_handle();
		let indices = vec![
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&indices), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_replay(indices)))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path"),
						PlayMode::Replay,
					)]));
				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();

		fn assert_replay(indices: Vec<AnimationNodeIndex>) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_is_playing().return_const(false);
				mock.expect_repeat().never().return_const(());
				for index in indices.clone() {
					mock.expect_replay()
						.times(1)
						.with(eq(index))
						.return_const(());
				}
				mock.expect_stop_animation().never().return_const(());
			}
		}
	}

	#[test]
	fn play_all_animations() {
		let handle = new_handle();
		let indices = vec![
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
			AnimationNodeIndex::new(4),
			AnimationNodeIndex::new(5),
			AnimationNodeIndex::new(6),
			AnimationNodeIndex::new(7),
			AnimationNodeIndex::new(8),
			AnimationNodeIndex::new(9),
			AnimationNodeIndex::new(10),
			AnimationNodeIndex::new(11),
			AnimationNodeIndex::new(12),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([
				(
					AnimationPath::from("my/path/high/1"),
					(_Animations::from(&indices[0..=1]), 0),
				),
				(
					AnimationPath::from("my/path/high/2"),
					(_Animations::from(&indices[2..=3]), 0),
				),
				(
					AnimationPath::from("my/path/medium/1"),
					(_Animations::from(&indices[4..=5]), 0),
				),
				(
					AnimationPath::from("my/path/medium/2"),
					(_Animations::from(&indices[6..=7]), 0),
				),
				(
					AnimationPath::from("my/path/low/1"),
					(_Animations::from(&indices[8..=9]), 0),
				),
				(
					AnimationPath::from("my/path/low/2"),
					(_Animations::from(&indices[10..=11]), 0),
				),
			]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat(indices)))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/high/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/high/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Medium))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/medium/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/medium/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Low))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/low/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/low/2"), PlayMode::Repeat),
					]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();

		fn assert_repeat(indices: Vec<AnimationNodeIndex>) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_is_playing().return_const(false);
				mock.expect_replay().never().return_const(());
				for index in indices.clone() {
					mock.expect_repeat()
						.times(1)
						.with(eq(index))
						.return_const(());
				}
				mock.expect_stop_animation().return_const(());
			}
		}
	}

	#[test]
	fn do_not_play_animation_which_is_playing() {
		let handle = new_handle();
		let indices = vec![
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&indices), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_not_playing(indices)))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));

				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path"),
						PlayMode::Repeat,
					)]));
				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();

		fn assert_not_playing(
			indices: Vec<AnimationNodeIndex>,
		) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				for i in indices.clone() {
					mock.expect_is_playing()
						.times(1)
						.with(eq(i))
						.return_const(true);
				}
				mock.expect_replay().never().return_const(());
				mock.expect_repeat().never().return_const(());
				mock.expect_stop_animation().never().return_const(());
			}
		}
	}

	#[test]
	fn stop_playing_animation_not_returned_in_dispatcher() {
		let handle = new_handle();
		let indices = vec![
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&indices), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_stop(indices)))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));

				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();

		fn assert_stop(indices: Vec<AnimationNodeIndex>) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_is_playing().return_const(false);
				mock.expect_replay().return_const(());
				mock.expect_repeat().return_const(());
				for i in indices.clone() {
					mock.expect_stop_animation()
						.times(1)
						.with(eq(i))
						.return_const(());
				}
			}
		}
	}

	#[test]
	fn play_animation_only_once() {
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&vec![AnimationNodeIndex::new(1)]), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat_once))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path"),
						PlayMode::Repeat,
					)]));
				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle),
		));

		app.update();
		app.update();

		fn assert_repeat_once(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat().times(1).return_const(());
			mock.expect_stop_animation().never().return_const(());
		}
	}

	#[test]
	fn play_animation_again_after_dispatcher_mutably_dereferenced() {
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&vec![AnimationNodeIndex::new(1)]), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat_twice))
			.id();
		let agent = app
			.world_mut()
			.spawn((
				_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
					mock.expect_animation_players()
						.return_const(_Iter::from([animation_player]));
					mock.expect_get_active_animations()
						.with(eq(AnimationPriority::High))
						.return_const(leak_iterator(vec![Animation::new(
							AnimationPath::from("my/path"),
							PlayMode::Repeat,
						)]));
					mock.expect_get_active_animations::<AnimationPriority>()
						.return_const(leak_iterator(vec![]));
				}),
				lookup,
				_GraphComponent(handle),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_AnimationDispatch>()
			.unwrap()
			.deref_mut();
		app.update();

		fn assert_repeat_twice(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat().times(2).return_const(());
			mock.expect_stop_animation().never().return_const(());
		}
	}

	#[test]
	fn mask_depending_on_priority() {
		let handle = new_handle();
		let indices = [
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
			AnimationNodeIndex::new(4),
			AnimationNodeIndex::new(5),
			AnimationNodeIndex::new(6),
			AnimationNodeIndex::new(7),
			AnimationNodeIndex::new(8),
			AnimationNodeIndex::new(9),
			AnimationNodeIndex::new(10),
			AnimationNodeIndex::new(11),
			AnimationNodeIndex::new(12),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([
				(
					AnimationPath::from("my/path/high/1"),
					(_Animations::from(&indices[0..=1]), 0b000001),
				),
				(
					AnimationPath::from("my/path/high/2"),
					(_Animations::from(&indices[2..=3]), 0b000010),
				),
				(
					AnimationPath::from("my/path/medium/1"),
					(_Animations::from(&indices[4..=5]), 0b000100),
				),
				(
					AnimationPath::from("my/path/medium/2"),
					(_Animations::from(&indices[6..=7]), 0b001000),
				),
				(
					AnimationPath::from("my/path/low/1"),
					(_Animations::from(&indices[8..=9]), 0b010000),
				),
				(
					AnimationPath::from("my/path/low/2"),
					(_Animations::from(&indices[10..=11]), 0b100000),
				),
			]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app.world_mut().spawn(_AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/high/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/high/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Medium))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/medium/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/medium/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Low))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/low/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/low/2"), PlayMode::Repeat),
					]));
			}),
			lookup,
			_GraphComponent(handle.clone()),
		));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(&handle)
			.unwrap();
		let masks = &indices
			.iter()
			.map(|i| graph.nodes.get(&i.index()).unwrap().mask)
			.collect::<Vec<_>>();
		// each priority has 2 assets each with 2 animations
		//   -> 4 animations masked by higher priority mask per priority
		let expected = &std::iter::repeat_n(0b000000, 4)
			.chain(std::iter::repeat_n(0b000011, 4))
			.chain(std::iter::repeat_n(0b001111, 4))
			.collect::<Vec<_>>();
		assert_eq!(
			expected,
			masks,
			"\n  left bits: {}\n right bits: {}",
			binary_str!(expected),
			binary_str!(masks)
		);
	}

	#[test]
	fn unmask_depending_on_priority() {
		let handle = new_handle();
		let indices = [
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
			AnimationNodeIndex::new(4),
			AnimationNodeIndex::new(5),
			AnimationNodeIndex::new(6),
			AnimationNodeIndex::new(7),
			AnimationNodeIndex::new(8),
			AnimationNodeIndex::new(9),
			AnimationNodeIndex::new(10),
			AnimationNodeIndex::new(11),
			AnimationNodeIndex::new(12),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([
				(
					AnimationPath::from("my/path/high/1"),
					(_Animations::from(&indices[0..=1]), 0b000001),
				),
				(
					AnimationPath::from("my/path/high/2"),
					(_Animations::from(&indices[2..=3]), 0b000010),
				),
				(
					AnimationPath::from("my/path/medium/1"),
					(_Animations::from(&indices[4..=5]), 0b000100),
				),
				(
					AnimationPath::from("my/path/medium/2"),
					(_Animations::from(&indices[6..=7]), 0b001000),
				),
				(
					AnimationPath::from("my/path/low/1"),
					(_Animations::from(&indices[8..=9]), 0b010000),
				),
				(
					AnimationPath::from("my/path/low/2"),
					(_Animations::from(&indices[10..=11]), 0b100000),
				),
			]),
		};
		let initial_mask = 0b111111;
		let mut app = setup!(&lookup, &handle, initial_mask);
		let animation_player = app.world_mut().spawn(_AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/high/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/high/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Medium))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/medium/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/medium/2"), PlayMode::Repeat),
					]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Low))
					.return_const(leak_iterator(vec![
						Animation::new(AnimationPath::from("my/path/low/1"), PlayMode::Repeat),
						Animation::new(AnimationPath::from("my/path/low/2"), PlayMode::Repeat),
					]));
			}),
			lookup,
			_GraphComponent(handle.clone()),
		));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(&handle)
			.unwrap();
		let masks = &indices
			.iter()
			.map(|i| graph.nodes.get(&i.index()).unwrap().mask)
			.collect::<Vec<_>>();
		// each asset has 2 animations
		//   -> 2 animations masked per asset
		let expected = &std::iter::repeat_n(0b111110, 2)
			.chain(std::iter::repeat_n(0b111101, 2))
			.chain(std::iter::repeat_n(0b111011, 2))
			.chain(std::iter::repeat_n(0b110111, 2))
			.chain(std::iter::repeat_n(0b101111, 2))
			.chain(std::iter::repeat_n(0b011111, 2))
			.collect::<Vec<_>>();
		assert_eq!(
			expected,
			masks,
			"\n  left bits: {}\n right bits: {}",
			binary_str!(expected),
			binary_str!(masks)
		);
	}

	#[test]
	fn set_mask_for_already_playing_animation() {
		let handle = new_handle();
		let indices = [
			AnimationNodeIndex::new(1),
			AnimationNodeIndex::new(2),
			AnimationNodeIndex::new(3),
			AnimationNodeIndex::new(4),
		];
		let lookup = AnimationLookup {
			animations: HashMap::from([
				(
					AnimationPath::from("my/path/hig"),
					(_Animations::from(&indices[0..=1]), 0b000001),
				),
				(
					AnimationPath::from("my/path/med"),
					(_Animations::from(&indices[2..=3]), 0b000111),
				), // wants to play on high masks (..11)
			]),
		};
		let initial_mask = 0b111111;
		let mut app = setup!(&lookup, &handle, initial_mask);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(|mock| {
				mock.expect_is_playing()
					.with(eq(indices[2]))
					.return_const(true);
				mock.expect_is_playing()
					.with(eq(indices[3]))
					.return_const(true);
				mock.expect_is_playing().return_const(false);
				mock.expect_repeat().return_const(());
				mock.expect_replay().return_const(());
				mock.expect_stop_animation().return_const(());
			}))
			.id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::High))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path/hig"),
						PlayMode::Repeat,
					)]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Medium))
					.return_const(leak_iterator(vec![Animation::new(
						AnimationPath::from("my/path/med"),
						PlayMode::Repeat,
					)]));
				mock.expect_get_active_animations()
					.with(eq(AnimationPriority::Low))
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle.clone()),
		));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(&handle)
			.unwrap();
		let masks = &indices
			.iter()
			.map(|i| graph.nodes.get(&i.index()).unwrap().mask)
			.collect::<Vec<_>>();
		// each asset has 2 animations
		//   -> 2 animations masked per asset
		let expected = &std::iter::repeat_n(0b111110, 2)
			.chain(std::iter::repeat_n(0b111001, 2))
			.collect::<Vec<_>>();
		assert_eq!(
			expected,
			masks,
			"\n  left bits: {}\n right bits: {}",
			binary_str!(expected),
			binary_str!(masks)
		);
	}

	#[test]
	fn completely_mask_animations_not_returned_by_dispatcher() {
		let handle = new_handle();
		let indices = vec![AnimationNodeIndex::new(1), AnimationNodeIndex::new(2)];
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(_Animations::from(&indices), 0),
			)]),
		};
		let mut app = setup!(&lookup, &handle);
		let animation_player = app.world_mut().spawn(_AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_AnimationDispatch::new().with_mock(|mock: &mut Mock_AnimationDispatch| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));

				mock.expect_get_active_animations::<AnimationPriority>()
					.return_const(leak_iterator(vec![]));
			}),
			lookup,
			_GraphComponent(handle.clone()),
		));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(&handle)
			.unwrap();
		let masks = &indices
			.iter()
			.map(|i| graph.nodes.get(&i.index()).unwrap().mask)
			.collect::<Vec<_>>();
		let expected = &vec![AnimationMask::MAX, AnimationMask::MAX];
		assert_eq!(
			expected,
			masks,
			"\n  left bits: {}\n right bits: {}",
			binary_str!(expected),
			binary_str!(masks)
		);
	}
}
