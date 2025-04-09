use crate::{AnimationData, traits::asset_server::animation_graph::GetNodeMut};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{iterate::Iterate, thread_safe::ThreadSafe},
};
use std::{any::type_name, marker::PhantomData};

impl<T> MaskAnimationNodes for T where T: Component {}

pub(crate) trait MaskAnimationNodes: Component + Sized {
	fn mask_animation_nodes(
		graphs: ResMut<Assets<AnimationGraph>>,
		animation_data: Res<AnimationData<Self>>,
	) -> Result<(), NoGraphForAgent<Self>> {
		mask_animation_nodes(graphs, animation_data)
	}
}

fn mask_animation_nodes<TAgent, TGraph, TAnimations>(
	mut graphs: ResMut<Assets<TGraph>>,
	animation_data: Res<AnimationData<TAgent, TGraph, TAnimations>>,
) -> Result<(), NoGraphForAgent<TAgent>>
where
	TAgent: Component,
	TGraph: Asset + GetNodeMut,
	TAnimations: ThreadSafe,
	for<'a> TAnimations: Iterate<'a, TItem = &'a AnimationNodeIndex>,
{
	let Some(graph) = graphs.get_mut(&animation_data.graph) else {
		return Err(NoGraphForAgent(PhantomData));
	};

	for (animations, _) in animation_data.animations.values() {
		for index in animations.iterate() {
			let Some(animation) = graph.get_node_mut(*index) else {
				continue;
			};

			animation.add_mask(AnimationMask::MAX);
		}
	}

	Ok(())
}

#[derive(Debug, PartialEq)]
pub(crate) struct NoGraphForAgent<TAgent>(PhantomData<TAgent>);

impl<TAgent> From<NoGraphForAgent<TAgent>> for Error {
	fn from(_: NoGraphForAgent<TAgent>) -> Self {
		Error {
			msg: format!(
				"{}: Does not have any `AnimationData`",
				type_name::<TAgent>()
			),
			lvl: Level::Error,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::AnimationData;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{animation::AnimationAsset, load_asset::Path},
	};
	use std::{collections::HashMap, slice::Iter};
	use uuid::Uuid;

	struct _Animations(Vec<AnimationNodeIndex>);

	impl<'a> Iterate<'a> for _Animations {
		type TItem = &'a AnimationNodeIndex;
		type TIter = Iter<'a, AnimationNodeIndex>;

		fn iterate(&'a self) -> Iter<'a, AnimationNodeIndex> {
			self.0.iter()
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

	fn unique_asset() -> AnimationAsset {
		AnimationAsset::Path(Path::from(Uuid::new_v4().to_string()))
	}

	fn setup<TAgent>(animations: Vec<_Animations>, graph_handle: &Handle<_Graph>) -> App
	where
		TAgent: Component,
	{
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();
		let mut graph = _Graph::default();

		for animations in &animations {
			for animation in animations.iterate() {
				graph
					.nodes
					.insert(animation.index(), AnimationGraphNode::default());
			}
		}

		graphs.insert(graph_handle, graph);
		app.insert_resource(AnimationData::<TAgent, _Graph, _Animations>::new(
			graph_handle.clone(),
			HashMap::from_iter(
				animations
					.into_iter()
					.map(|animation| (unique_asset(), (animation, AnimationMask::default()))),
			),
		));
		app.insert_resource(graphs);

		app
	}

	#[test]
	fn set_all_animations_to_being_fully_masked() -> Result<(), RunSystemError> {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		let animations = vec![
			_Animations(vec![AnimationNodeIndex::new(1), AnimationNodeIndex::new(2)]),
			_Animations(vec![AnimationNodeIndex::new(3), AnimationNodeIndex::new(4)]),
		];
		let graph = &new_handle();
		let mut app = setup::<_Agent>(animations, graph);

		let result = app
			.world_mut()
			.run_system_once(mask_animation_nodes::<_Agent, _Graph, _Animations>)?;

		let graph = app.world().resource::<Assets<_Graph>>().get(graph).unwrap();
		let masks = graph
			.nodes
			.values()
			.map(|node| node.mask)
			.collect::<Vec<_>>();
		assert_eq!(
			(
				Ok(()),
				vec![
					AnimationMask::MAX,
					AnimationMask::MAX,
					AnimationMask::MAX,
					AnimationMask::MAX,
				],
			),
			(result, masks)
		);
		Ok(())
	}

	#[test]
	fn do_not_set_animations_masks_of_other_agent() -> Result<(), RunSystemError> {
		#[derive(Component)]
		struct _Agent;

		#[derive(Component)]
		struct _OtherAgent;

		let animations = vec![
			_Animations(vec![AnimationNodeIndex::new(1), AnimationNodeIndex::new(2)]),
			_Animations(vec![AnimationNodeIndex::new(3), AnimationNodeIndex::new(4)]),
		];
		let graph = &new_handle();
		let mut app = setup::<_OtherAgent>(animations, graph);
		app.insert_resource(AnimationData::<_Agent, _Graph, _Animations>::new(
			new_handle(),
			HashMap::default(),
		));

		_ = app
			.world_mut()
			.run_system_once(mask_animation_nodes::<_Agent, _Graph, _Animations>)?;

		let graph = app.world().resource::<Assets<_Graph>>().get(graph).unwrap();
		let masks = graph
			.nodes
			.values()
			.map(|node| node.mask)
			.collect::<Vec<_>>();
		assert_eq!(
			vec![
				AnimationMask::default(),
				AnimationMask::default(),
				AnimationMask::default(),
				AnimationMask::default(),
			],
			masks
		);
		Ok(())
	}

	#[test]
	fn return_no_graph_error() -> Result<(), RunSystemError> {
		#[derive(Component, Debug, PartialEq)]
		struct _Agent;

		let mut app = setup::<_Agent>(vec![], &new_handle());
		app.world_mut()
			.resource_mut::<AnimationData<_Agent, _Graph, _Animations>>()
			.graph = new_handle();

		let error = app
			.world_mut()
			.run_system_once(mask_animation_nodes::<_Agent, _Graph, _Animations>)?;

		assert_eq!(Err(NoGraphForAgent(PhantomData::<_Agent>)), error);
		Ok(())
	}
}
