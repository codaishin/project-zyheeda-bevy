use crate::{
	components::animation_lookup::AnimationLookup,
	traits::asset_server::animation_graph::GetNodeMut,
};
use bevy::prelude::*;
use common::traits::{
	iterate::Iterate,
	thread_safe::ThreadSafe,
	wrap_handle::{UnwrapHandle, WrapHandle},
};

impl<T> MaskAnimationNodes for T where T: Component {}

pub(crate) trait MaskAnimationNodes: Component + Sized {
	fn mask_animation_nodes(
		graphs: ResMut<Assets<AnimationGraph>>,
		agents: Query<(&AnimationGraphHandle, &AnimationLookup), Added<Self>>,
	) {
		mask_animation_nodes(graphs, agents)
	}
}

fn mask_animation_nodes<TAgent, TGraph, TAnimations>(
	mut graphs: ResMut<Assets<TGraph>>,
	agents: Query<(&TGraph::TComponent, &AnimationLookup<TAnimations>), Added<TAgent>>,
) where
	TAgent: Component,
	TGraph: Asset + GetNodeMut + WrapHandle,
	TAnimations: ThreadSafe,
	for<'a> TAnimations: Iterate<'a, TItem = &'a AnimationNodeIndex>,
{
	for (graph_component, lookup) in &agents {
		let handle = graph_component.unwrap();
		let Some(graph) = graphs.get_mut(handle) else {
			continue;
		};

		for (animations, _) in lookup.animations.values() {
			for index in animations.iterate() {
				let Some(animation) = graph.get_node_mut(*index) else {
					continue;
				};

				animation.add_mask(AnimationMask::MAX);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::unique_animation_asset;
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::wrap_handle::{UnwrapHandle, WrapHandle},
	};
	use std::{collections::HashMap, slice::Iter};

	struct _Animations(Vec<AnimationNodeIndex>);

	impl<'a> Iterate<'a> for _Animations {
		type TItem = &'a AnimationNodeIndex;
		type TIter = Iter<'a, AnimationNodeIndex>;

		fn iterate(&'a self) -> Iter<'a, AnimationNodeIndex> {
			self.0.iter()
		}
	}

	#[derive(Debug, Clone, TypePath, Asset, Default)]
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

	#[derive(Component, Debug, PartialEq)]
	struct _Agent;

	fn setup(lookup: &AnimationLookup<_Animations>, graph_handle: &Handle<_Graph>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();
		let mut graph = _Graph::default();

		for (animations, mask) in lookup.animations.values() {
			for animation in animations.iterate() {
				graph.nodes.insert(
					animation.index(),
					AnimationGraphNode {
						mask: *mask,
						..default()
					},
				);
			}
		}

		graphs.insert(graph_handle, graph);
		app.insert_resource(graphs);
		app.add_systems(Update, mask_animation_nodes::<_Agent, _Graph, _Animations>);

		app
	}

	#[test]
	fn set_all_animations_to_being_fully_masked() {
		let lookup = AnimationLookup {
			animations: HashMap::from([
				(
					unique_animation_asset(),
					(
						_Animations(vec![AnimationNodeIndex::new(1), AnimationNodeIndex::new(2)]),
						AnimationMask::default(),
					),
				),
				(
					unique_animation_asset(),
					(
						_Animations(vec![AnimationNodeIndex::new(3), AnimationNodeIndex::new(4)]),
						AnimationMask::default(),
					),
				),
			]),
		};
		let handle = &new_handle();
		let mut app = setup(&lookup, handle);
		app.world_mut()
			.spawn((_Agent, lookup, _GraphComponent(handle.clone())));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(handle)
			.unwrap();
		let masks = graph
			.nodes
			.values()
			.map(|node| node.mask)
			.collect::<Vec<_>>();
		assert_eq!(
			vec![
				AnimationMask::MAX,
				AnimationMask::MAX,
				AnimationMask::MAX,
				AnimationMask::MAX,
			],
			masks
		);
	}

	#[test]
	fn act_only_once() {
		let lookup = AnimationLookup {
			animations: HashMap::from([(
				unique_animation_asset(),
				(
					_Animations(vec![AnimationNodeIndex::new(1)]),
					AnimationMask::default(),
				),
			)]),
		};
		let handle = &new_handle();
		let mut app = setup(&lookup, handle);
		app.world_mut()
			.spawn((_Agent, lookup, _GraphComponent(handle.clone())));

		app.update();
		let mut graphs = app.world_mut().resource_mut::<Assets<_Graph>>();
		let graph = graphs.get_mut(handle).unwrap();
		graph.nodes.get_mut(&1).unwrap().mask = AnimationMask::default();
		app.update();

		let graph = app
			.world()
			.resource::<Assets<_Graph>>()
			.get(handle)
			.unwrap();
		let masks = graph
			.nodes
			.values()
			.map(|node| node.mask)
			.collect::<Vec<_>>();
		assert_eq!(vec![AnimationMask::default(),], masks);
	}
}
