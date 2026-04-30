use crate::{
	components::{
		animation_dispatch::AnimationPlayers,
		animation_lookup::{AnimationClips, AnimationLookup},
		current_forward_pitch::CurrentForwardPitch,
	},
	traits::{GetAllActiveAnimations, asset_server::animation_graph::GetNodeMut},
};
use bevy::prelude::*;
use common::traits::{
	handles_animations::DirForwardPitch,
	wrap_handle::{GetHandle, WrapHandle},
};

impl<T> SetPitchAnimationWeights for T where T: Component + GetAllActiveAnimations {}

pub(crate) trait SetPitchAnimationWeights:
	Component + GetAllActiveAnimations + Sized
{
	fn set_pitch_animation_weights(
		graphs: ResMut<Assets<AnimationGraph>>,
		agents: Query<(
			&Self,
			&AnimationPlayers,
			&CurrentForwardPitch,
			&AnimationLookup,
		)>,
		players: Query<&AnimationGraphHandle>,
	) {
		set_pitch_animation_weights(graphs, agents, players)
	}
}

/// Using a min neutral animation weight to prevent odd deformations.
const MIN_NEUTRAL_WEIGHT: f32 = 1e-6;

macro_rules! set {
	($value:expr, $graph:expr, $($clip:expr),+ $(,)?) => {
		$({
			if let Some(animation) = $graph.get_node_mut($clip) {
				animation.weight = $value;
			}
		})+
	};
}

macro_rules! blend {
	($pitch:expr, $graph:expr, $neutral_clip:expr, $pitched_clip:expr) => {{
		let (max_pitch, pitched_clip) = $pitched_clip;
		let offset = **$pitch / *max_pitch;
		let neutral_weight = (1. - offset).clamp(MIN_NEUTRAL_WEIGHT, 1.);
		let pitched_weight = offset.clamp(0., 1.);

		set!(neutral_weight, $graph, $neutral_clip);
		set!(pitched_weight, $graph, pitched_clip);
	}};
}

fn set_pitch_animation_weights<TDispatch, TGraph>(
	mut graphs: ResMut<Assets<TGraph>>,
	agents: Query<(
		&TDispatch,
		&AnimationPlayers,
		&CurrentForwardPitch,
		&AnimationLookup,
	)>,
	players: Query<&TGraph::TComponent>,
) where
	TDispatch: Component + GetAllActiveAnimations,
	TGraph: Asset + GetNodeMut + WrapHandle,
{
	for (dispatch, animation_players, CurrentForwardPitch(pitch), lookup) in &agents {
		for entity in animation_players.iter() {
			let Ok(player) = players.get(entity) else {
				continue;
			};
			let Some(graph) = graphs.get_mut(player.get_handle()) else {
				continue;
			};

			for animation in dispatch.get_all_active_animations() {
				let Some(data) = lookup.animations.get(animation) else {
					continue;
				};
				let AnimationClips::PitchedForward(pitched_clips) = &data.animation_clips else {
					continue;
				};

				match pitch {
					None => {
						set!(1., graph, pitched_clips.neutral);
						set!(0., graph, pitched_clips.up.1, pitched_clips.down.1);
					}
					Some(DirForwardPitch::Up(pitch)) => {
						blend!(pitch, graph, pitched_clips.neutral, pitched_clips.up);
						set!(0., graph, pitched_clips.down.1);
					}
					Some(DirForwardPitch::Down(pitch)) => {
						blend!(pitch, graph, pitched_clips.neutral, pitched_clips.down);
						set!(0., graph, pitched_clips.up.1);
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::{
		animation_dispatch::AnimationPlayerOf,
		animation_lookup::{AnimationClips, AnimationLookupData, PitchedForwardIndices},
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{
			handles_animations::{AnimationKey, ForwardPitch, SkillAnimation},
			iterate::Iterate,
			wrap_handle::{GetHandle, WrapHandle},
		},
	};
	use std::{collections::HashMap, slice::Iter};
	use test_case::test_case;
	use testing::{SingleThreadedApp, assert_eq_approx, new_handle};

	#[derive(Component)]
	struct _Dispatch {
		animations: Vec<AnimationKey>,
	}

	impl GetAllActiveAnimations for _Dispatch {
		type TIter<'a>
			= Iter<'a, AnimationKey>
		where
			Self: 'a;

		fn get_all_active_animations(&self) -> Self::TIter<'_> {
			self.animations.iter()
		}
	}

	#[derive(Asset, TypePath, Default)]
	struct _Graph {
		nodes: HashMap<usize, AnimationGraphNode>,
	}

	impl GetNodeMut for _Graph {
		fn get_node_mut(
			&mut self,
			animation: AnimationNodeIndex,
		) -> Option<&mut AnimationGraphNode> {
			self.nodes.get_mut(&animation.index())
		}
	}

	impl WrapHandle for _Graph {
		type TComponent = _GraphComponent;

		fn wrap_handle(handle: Handle<Self>) -> Self::TComponent {
			_GraphComponent(handle)
		}
	}

	#[derive(Component)]
	struct _GraphComponent(Handle<_Graph>);

	impl GetHandle for _GraphComponent {
		type TAsset = _Graph;

		fn get_handle(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	fn setup(
		lookup: &AnimationLookup,
		weights: HashMap<usize, f32>,
		graph_handle: &Handle<_Graph>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();
		let mut graph = _Graph::default();

		for data in lookup.animations.values() {
			for animation in data.animation_clips.iterate() {
				graph.nodes.insert(
					animation.index(),
					AnimationGraphNode {
						weight: weights.get(&animation.index()).copied().unwrap_or(0.),
						..default()
					},
				);
			}
		}

		_ = graphs.insert(graph_handle, graph);
		app.insert_resource(graphs);
		app.add_systems(Update, set_pitch_animation_weights::<_Dispatch, _Graph>);

		app
	}

	#[test_case(None, [1., 0., 0.]; "neutral")]
	#[test_case(Some(DirForwardPitch::Up(ForwardPitch::MAX)), [MIN_NEUTRAL_WEIGHT, 1., 0.]; "up")]
	#[test_case(Some(DirForwardPitch::Down(ForwardPitch::MAX)), [MIN_NEUTRAL_WEIGHT, 0., 1.]; "down")]
	fn apply_full_weights(pitch: Option<DirForwardPitch>, expected_weights: [f32; 3]) {
		let initial_weights = || {
			expected_weights
				.map(|weight| match weight {
					0. => 1.,
					_ => 0.,
				})
				.into_iter()
				.enumerate()
		};
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationKey::Skill {
					slot: SlotKey(11),
					animation: SkillAnimation::Shoot,
				},
				AnimationLookupData {
					animation_clips: AnimationClips::PitchedForward(PitchedForwardIndices {
						neutral: AnimationNodeIndex::new(0),
						up: (ForwardPitch::MAX, AnimationNodeIndex::new(1)),
						down: (ForwardPitch::MAX, AnimationNodeIndex::new(2)),
					}),
					..default()
				},
			)]),
			..default()
		};
		let weights = HashMap::from_iter(initial_weights());
		let mut app = setup(&lookup, weights, &handle);
		let agent = app
			.world_mut()
			.spawn((
				_Dispatch {
					animations: vec![AnimationKey::Skill {
						slot: SlotKey(11),
						animation: SkillAnimation::Shoot,
					}],
				},
				CurrentForwardPitch(pitch),
				lookup,
			))
			.id();
		app.world_mut()
			.spawn((_GraphComponent(handle.clone()), AnimationPlayerOf(agent)));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
			],
			f32::EPSILON
		);
	}

	#[test_case(Some(DirForwardPitch::Up(ForwardPitch::try_from(0.3).unwrap())), [0.7, 0.3, 0.]; "up")]
	#[test_case(Some(DirForwardPitch::Down(ForwardPitch::try_from(0.3).unwrap())), [0.7, 0., 0.3]; "down")]
	fn apply_blended_weights_when_configured_with_max_pitch(
		pitch: Option<DirForwardPitch>,
		expected_weights: [f32; 3],
	) {
		let initial_weights = || {
			expected_weights
				.map(|weight| match weight {
					0. => 1.,
					_ => 0.,
				})
				.into_iter()
				.enumerate()
		};
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationKey::Skill {
					slot: SlotKey(11),
					animation: SkillAnimation::Shoot,
				},
				AnimationLookupData {
					animation_clips: AnimationClips::PitchedForward(PitchedForwardIndices {
						neutral: AnimationNodeIndex::new(0),
						up: (ForwardPitch::MAX, AnimationNodeIndex::new(1)),
						down: (ForwardPitch::MAX, AnimationNodeIndex::new(2)),
					}),
					..default()
				},
			)]),
			..default()
		};
		let weights = HashMap::from_iter(initial_weights());
		let mut app = setup(&lookup, weights, &handle);
		let agent = app
			.world_mut()
			.spawn((
				_Dispatch {
					animations: vec![AnimationKey::Skill {
						slot: SlotKey(11),
						animation: SkillAnimation::Shoot,
					}],
				},
				CurrentForwardPitch(pitch),
				lookup,
			))
			.id();
		app.world_mut()
			.spawn((_GraphComponent(handle.clone()), AnimationPlayerOf(agent)));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
			],
			f32::EPSILON
		);
	}

	#[test_case(Some(DirForwardPitch::Up(ForwardPitch::try_from(0.2).unwrap())), [0.75, 0.25, 0.]; "up")]
	#[test_case(Some(DirForwardPitch::Down(ForwardPitch::try_from(0.2).unwrap())), [0.75, 0., 0.25]; "down")]
	fn apply_blended_weights_when_configured_with_less_than_max_pitch(
		pitch: Option<DirForwardPitch>,
		expected_weights: [f32; 3],
	) {
		let initial_weights = || {
			expected_weights
				.map(|weight| match weight {
					0. => 1.,
					_ => 0.,
				})
				.into_iter()
				.enumerate()
		};
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationKey::Skill {
					slot: SlotKey(11),
					animation: SkillAnimation::Shoot,
				},
				AnimationLookupData {
					animation_clips: AnimationClips::PitchedForward(PitchedForwardIndices {
						neutral: AnimationNodeIndex::new(0),
						up: (
							ForwardPitch::try_from(0.8).unwrap(),
							AnimationNodeIndex::new(1),
						),
						down: (
							ForwardPitch::try_from(0.8).unwrap(),
							AnimationNodeIndex::new(2),
						),
					}),
					..default()
				},
			)]),
			..default()
		};
		let weights = HashMap::from_iter(initial_weights());
		let mut app = setup(&lookup, weights, &handle);
		let agent = app
			.world_mut()
			.spawn((
				_Dispatch {
					animations: vec![AnimationKey::Skill {
						slot: SlotKey(11),
						animation: SkillAnimation::Shoot,
					}],
				},
				CurrentForwardPitch(pitch),
				lookup,
			))
			.id();
		app.world_mut()
			.spawn((_GraphComponent(handle.clone()), AnimationPlayerOf(agent)));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
			],
			f32::EPSILON
		);
	}

	#[test_case(Some(DirForwardPitch::Up(ForwardPitch::try_from(0.9).unwrap())), [MIN_NEUTRAL_WEIGHT, 1., 0.]; "up")]
	#[test_case(Some(DirForwardPitch::Down(ForwardPitch::try_from(0.9).unwrap())), [MIN_NEUTRAL_WEIGHT, 0., 1.]; "down")]
	fn apply_clamped_blended_weights_when_configured_with_less_than_max_pitch(
		pitch: Option<DirForwardPitch>,
		expected_weights: [f32; 3],
	) {
		let initial_weights = || {
			expected_weights
				.map(|weight| match weight {
					0. => 1.,
					_ => 0.,
				})
				.into_iter()
				.enumerate()
		};
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationKey::Skill {
					slot: SlotKey(11),
					animation: SkillAnimation::Shoot,
				},
				AnimationLookupData {
					animation_clips: AnimationClips::PitchedForward(PitchedForwardIndices {
						neutral: AnimationNodeIndex::new(0),
						up: (
							ForwardPitch::try_from(0.8).unwrap(),
							AnimationNodeIndex::new(1),
						),
						down: (
							ForwardPitch::try_from(0.8).unwrap(),
							AnimationNodeIndex::new(2),
						),
					}),
					..default()
				},
			)]),
			..default()
		};
		let weights = HashMap::from_iter(initial_weights());
		let mut app = setup(&lookup, weights, &handle);
		let agent = app
			.world_mut()
			.spawn((
				_Dispatch {
					animations: vec![AnimationKey::Skill {
						slot: SlotKey(11),
						animation: SkillAnimation::Shoot,
					}],
				},
				CurrentForwardPitch(pitch),
				lookup,
			))
			.id();
		app.world_mut()
			.spawn((_GraphComponent(handle.clone()), AnimationPlayerOf(agent)));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
			],
			f32::EPSILON
		);
	}
}
