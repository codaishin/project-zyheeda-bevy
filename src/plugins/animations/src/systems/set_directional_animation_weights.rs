use crate::{
	components::animation_lookup::{AnimationLookup, Animations},
	traits::{AnimationPlayers, GetAllActiveAnimations, asset_server::animation_graph::GetNodeMut},
};
use bevy::prelude::*;
use common::traits::{
	animation::{Animation, GetMovementDirection},
	wrap_handle::{UnwrapHandle, WrapHandle},
};
use std::f32::consts::FRAC_PI_2;

impl<T> SetDirectionalAnimationWeights for T where
	T: Component + AnimationPlayers + GetAllActiveAnimations<Animation>
{
}

pub(crate) trait SetDirectionalAnimationWeights:
	Component + AnimationPlayers + GetAllActiveAnimations<Animation> + Sized
{
	fn set_directional_animation_weights<TMovementDirection>(
		graphs: ResMut<Assets<AnimationGraph>>,
		agents: Query<(
			&Self,
			&TMovementDirection,
			&GlobalTransform,
			&AnimationLookup,
		)>,
		players: Query<&AnimationGraphHandle>,
	) where
		TMovementDirection: Component + GetMovementDirection,
	{
		set_directional_animation_weights(graphs, agents, players)
	}
}

fn set_directional_animation_weights<TDispatch, TGraph, TMovementDirection>(
	mut graphs: ResMut<Assets<TGraph>>,
	agents: Query<(
		&TDispatch,
		&TMovementDirection,
		&GlobalTransform,
		&AnimationLookup,
	)>,
	players: Query<&TGraph::TComponent>,
) where
	TDispatch: Component + AnimationPlayers + GetAllActiveAnimations<Animation>,
	TGraph: Asset + GetNodeMut + WrapHandle,
	TMovementDirection: Component + GetMovementDirection,
{
	for (dispatch, direction, transform, lookup) in &agents {
		let Some(direction) = direction.movement_direction(transform) else {
			continue;
		};
		let forward = transform.forward();
		let backward = transform.back();
		let left = transform.left();
		let right = transform.right();

		for entity in dispatch.animation_players() {
			let Ok(player) = players.get(entity) else {
				continue;
			};
			let Some(graph) = graphs.get_mut(player.unwrap()) else {
				continue;
			};

			for animation in dispatch.get_all_active_animations() {
				let Some((animation, ..)) = lookup.animations.get(&animation.path) else {
					continue;
				};
				let &Animations::Directional(directions) = animation else {
					continue;
				};

				if let Some(animation) = graph.get_node_mut(directions.forward) {
					animation.weight = weight(forward, direction);
				}
				if let Some(animation) = graph.get_node_mut(directions.backward) {
					animation.weight = weight(backward, direction);
				}
				if let Some(animation) = graph.get_node_mut(directions.left) {
					animation.weight = weight(left, direction);
				}
				if let Some(animation) = graph.get_node_mut(directions.right) {
					animation.weight = weight(right, direction);
				}
			}
		}
	}
}

fn weight(body_direction: Dir3, move_direction: Dir3) -> f32 {
	let dot = body_direction.dot(*move_direction);

	if dot <= 0. {
		return 0.;
	}

	if dot >= 1. {
		return 1.;
	}

	let angle = dot.acos();
	1.0 - angle / FRAC_PI_2
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_lookup::{AnimationLookup, Animations, DirectionalIndices};
	use common::traits::{
		animation::{AnimationPath, PlayMode},
		iterate::Iterate,
		wrap_handle::{UnwrapHandle, WrapHandle},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{collections::HashMap, slice::Iter, vec::IntoIter};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp, assert_eq_approx, new_handle};

	#[derive(Component)]
	struct _Dispatch {
		players: Vec<Entity>,
		animations: Vec<Animation>,
	}

	impl AnimationPlayers for _Dispatch {
		type TIter = IntoIter<Entity>;

		fn animation_players(&self) -> Self::TIter {
			self.players.clone().into_iter()
		}
	}

	impl GetAllActiveAnimations<Animation> for _Dispatch {
		type TIter<'a>
			= Iter<'a, Animation>
		where
			Self: 'a,
			Animation: 'a;

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

		fn wrap(handle: Handle<Self>) -> Self::TComponent {
			_GraphComponent(handle)
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

	#[derive(Component, NestedMocks)]
	struct _Direction {
		mock: Mock_Direction,
	}

	#[automock]
	impl GetMovementDirection for _Direction {
		fn movement_direction(&self, transform: &GlobalTransform) -> Option<Dir3> {
			self.mock.movement_direction(transform)
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

		for (animations, _) in lookup.animations.values() {
			for animation in animations.iterate() {
				graph.nodes.insert(
					animation.index(),
					AnimationGraphNode {
						weight: weights.get(&animation.index()).copied().unwrap_or(0.),
						..default()
					},
				);
			}
		}

		graphs.insert(graph_handle, graph);
		app.insert_resource(graphs);
		app.add_systems(
			Update,
			set_directional_animation_weights::<_Dispatch, _Graph, _Direction>,
		);

		app
	}

	#[test_case(Dir3::NEG_Z, [1., 0., 0., 0.]; "forward")]
	#[test_case(Dir3::Z, [0., 1., 0., 0.]; "backward")]
	#[test_case(Dir3::NEG_X, [0., 0., 1., 0.]; "left")]
	#[test_case(Dir3::X, [0., 0., 0., 1.]; "right")]
	fn apply_weights_for_direction(direction: Dir3, expected_weights: [f32; 4]) {
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
				AnimationPath::from("my/path"),
				(
					Animations::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(0),
						backward: AnimationNodeIndex::new(1),
						left: AnimationNodeIndex::new(2),
						right: AnimationNodeIndex::new(3),
					}),
					0,
				),
			)]),
		};
		let weights = HashMap::from_iter(initial_weights());
		let mut app = setup(&lookup, weights, &handle);
		let player = app.world_mut().spawn(_GraphComponent(handle.clone())).id();
		app.world_mut().spawn((
			GlobalTransform::default(),
			_Dispatch {
				players: vec![player],
				animations: vec![Animation::new(
					AnimationPath::from("my/path"),
					PlayMode::Repeat,
				)],
			},
			_Direction::new().with_mock(|mock| {
				mock.expect_movement_direction()
					.return_const(Some(direction));
			}),
			lookup,
		));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
				graph.nodes.get(&3).unwrap().weight
			],
			f32::EPSILON
		);
	}

	#[test_case(Dir3::X, [1., 0., 0., 0.]; "forward")]
	#[test_case(Dir3::NEG_X, [0., 1., 0., 0.]; "backward")]
	#[test_case(Dir3::NEG_Z, [0., 0., 1., 0.]; "left")]
	#[test_case(Dir3::Z, [0., 0., 0., 1.]; "right")]
	fn looking_right_apply_weights_for_direction(direction: Dir3, expected_weights: [f32; 4]) {
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(
					Animations::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(0),
						backward: AnimationNodeIndex::new(1),
						left: AnimationNodeIndex::new(2),
						right: AnimationNodeIndex::new(3),
					}),
					0,
				),
			)]),
		};
		let mut app = setup(&lookup, HashMap::from([]), &handle);
		let player = app.world_mut().spawn(_GraphComponent(handle.clone())).id();
		app.world_mut().spawn((
			GlobalTransform::from(Transform::default().looking_to(Dir3::X, Vec3::Y)),
			_Dispatch {
				players: vec![player],
				animations: vec![Animation::new(
					AnimationPath::from("my/path"),
					PlayMode::Repeat,
				)],
			},
			_Direction::new().with_mock(|mock| {
				mock.expect_movement_direction()
					.return_const(Some(direction));
			}),
			lookup,
		));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
				graph.nodes.get(&3).unwrap().weight
			],
			f32::EPSILON
		);
	}

	#[test_case(Dir3::NEG_Z, [0.5, 0., 0.5, 0.]; "global forward")]
	#[test_case(Dir3::Z, [0., 0.5, 0., 0.5]; "global backward")]
	#[test_case(Dir3::NEG_X, [0., 0.5, 0.5, 0.]; "global left")]
	#[test_case(Dir3::X, [0.5, 0., 0., 0.5]; "global right")]
	fn looking_forward_right_apply_weights_for_direction(
		direction: Dir3,
		expected_weights: [f32; 4],
	) {
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(
					Animations::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(0),
						backward: AnimationNodeIndex::new(1),
						left: AnimationNodeIndex::new(2),
						right: AnimationNodeIndex::new(3),
					}),
					0,
				),
			)]),
		};
		let mut app = setup(&lookup, HashMap::from([]), &handle);
		let player = app.world_mut().spawn(_GraphComponent(handle.clone())).id();
		app.world_mut().spawn((
			GlobalTransform::from(
				Transform::default()
					.looking_to(Dir3::new(Vec3::new(1., 0., -1.)).unwrap(), Vec3::Y),
			),
			_Dispatch {
				players: vec![player],
				animations: vec![Animation::new(
					AnimationPath::from("my/path"),
					PlayMode::Repeat,
				)],
			},
			_Direction::new().with_mock(|mock| {
				mock.expect_movement_direction()
					.return_const(Some(direction));
			}),
			lookup,
		));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			expected_weights,
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
				graph.nodes.get(&3).unwrap().weight
			],
			f32::EPSILON
		);
	}

	#[test]
	fn use_correct_transform_to_determine_direction() {
		let handle = new_handle();
		let transform =
			GlobalTransform::from(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Vec3::Y));
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(
					Animations::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(0),
						backward: AnimationNodeIndex::new(1),
						left: AnimationNodeIndex::new(2),
						right: AnimationNodeIndex::new(3),
					}),
					0,
				),
			)]),
		};
		let mut app = setup(&lookup, HashMap::default(), &handle);
		let player = app.world_mut().spawn(_GraphComponent(handle.clone())).id();
		app.world_mut().spawn((
			transform,
			_Dispatch {
				players: vec![player],
				animations: vec![Animation::new(
					AnimationPath::from("my/path"),
					PlayMode::Repeat,
				)],
			},
			_Direction::new().with_mock(assert_being_called_with(transform)),
			lookup,
		));

		app.update();

		fn assert_being_called_with(transform: GlobalTransform) -> impl Fn(&mut Mock_Direction) {
			move |mock| {
				mock.expect_movement_direction()
					.times(1)
					.with(eq(transform))
					.return_const(None);
			}
		}
	}

	#[test]
	fn prevent_weight_nan_for_close_directions_round_error() {
		let handle = new_handle();
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				AnimationPath::from("my/path"),
				(
					Animations::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(0),
						backward: AnimationNodeIndex::new(1),
						left: AnimationNodeIndex::new(2),
						right: AnimationNodeIndex::new(3),
					}),
					0,
				),
			)]),
		};
		let mut app = setup(&lookup, HashMap::from([]), &handle);
		let player = app.world_mut().spawn(_GraphComponent(handle.clone())).id();
		app.world_mut().spawn((
			GlobalTransform::from(Transform::default().looking_to(
				// taken from production, when causing a NaN weight
				Dir3::new(Vec3::new(-0.039663047, -0.0, -0.9992132)).unwrap(),
				Vec3::Y,
			)),
			_Dispatch {
				players: vec![player],
				animations: vec![Animation::new(
					AnimationPath::from("my/path"),
					PlayMode::Repeat,
				)],
			},
			_Direction::new().with_mock(|mock| {
				mock.expect_movement_direction().return_const(Some(
					// taken from production, when causing a NaN weight
					Dir3::new(Vec3::new(-0.039663114, 0.0, -0.9992131)).unwrap(),
				));
			}),
			lookup,
		));

		app.update();

		let graphs = app.world().resource::<Assets<_Graph>>();
		let graph = graphs.get(&handle).unwrap();
		assert_eq_approx!(
			[1., 0., 0., 0.],
			[
				graph.nodes.get(&0).unwrap().weight,
				graph.nodes.get(&1).unwrap().weight,
				graph.nodes.get(&2).unwrap().weight,
				graph.nodes.get(&3).unwrap().weight
			],
			f32::EPSILON
		);
	}
}
