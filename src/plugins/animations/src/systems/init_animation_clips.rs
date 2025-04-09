use crate::{
	resource::{AnimationData, Animations},
	traits::LoadAnimationAssets,
};
use bevy::prelude::*;
use common::traits::animation::{AnimationMaskDefinition, GetAnimationDefinitions};
use std::collections::HashMap;

impl<TAgent> InitAnimationClips for TAgent
where
	TAgent: GetAnimationDefinitions + Sync + Send + 'static,
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskDefinition: From<&'a Self::TAnimationMask>,
{
}

pub(crate) trait InitAnimationClips
where
	Self: GetAnimationDefinitions + Sync + Send + Sized + 'static,
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskDefinition: From<&'a Self::TAnimationMask>,
{
	fn init_animation_clips<
		TAnimationGraph: Asset + Sync + Send + 'static,
		TServer: Resource + LoadAnimationAssets<TAnimationGraph, Animations>,
	>(
		mut commands: Commands,
		mut server: ResMut<TServer>,
		mut graphs: ResMut<Assets<TAnimationGraph>>,
	) {
		let masks = Self::animations();
		let animations = masks.keys().cloned().collect::<Vec<_>>();
		let (graph, new_clips) = server.load_animation_assets(animations);
		let graph = graphs.add(graph);

		let new_clips = new_clips.into_iter().filter_map(|(path, clip)| {
			let mask = masks.get(&path)?;
			Some((path, (clip, *mask)))
		});

		commands.insert_resource(AnimationData::<Self, TAnimationGraph>::new(
			graph,
			HashMap::from_iter(new_clips),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resource::AnimationData;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			animation::{AnimationAsset, AnimationMaskDefinition},
			iteration::{Iter, IterFinite},
			nested_mock::NestedMocks,
			thread_safe::ThreadSafe,
		},
	};
	use macros::NestedMocks;
	use mockall::automock;
	use std::collections::{HashMap, HashSet};

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[derive(Debug, PartialEq, Clone, TypePath, Asset)]
	struct _AnimationGraph;

	#[derive(Clone, Copy)]
	struct _Mask;

	impl IterFinite for _Mask {
		fn iterator() -> Iter<Self> {
			panic!("SHOULD NOT BE USED HERE")
		}

		fn next(_: &Iter<Self>) -> Option<Self> {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	impl From<&_Mask> for AnimationMask {
		fn from(_: &_Mask) -> Self {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	impl From<&_Mask> for AnimationMaskDefinition {
		fn from(_: &_Mask) -> Self {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	#[automock]
	impl LoadAnimationAssets<_AnimationGraph, Animations> for _Server {
		fn load_animation_assets(
			&mut self,
			animations: Vec<AnimationAsset>,
		) -> (_AnimationGraph, HashMap<AnimationAsset, Animations>) {
			self.mock.load_animation_assets(animations)
		}
	}

	fn setup<TAgent>(server: _Server) -> App
	where
		TAgent: GetAnimationDefinitions + ThreadSafe,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskDefinition: From<&'a TAgent::TAnimationMask>,
	{
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(server);
		app.insert_resource(Assets::<_AnimationGraph>::default());
		app.add_systems(
			Update,
			TAgent::init_animation_clips::<_AnimationGraph, _Server>,
		);

		app
	}

	#[test]
	fn store_animation_graph() {
		#[derive(Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<AnimationAsset, AnimationMask> {
				HashMap::default()
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.return_const((_AnimationGraph, HashMap::default()));
		}));

		app.update();

		let graphs = app.world().resource::<Assets<_AnimationGraph>>();
		let animation_data = app
			.world()
			.get_resource::<AnimationData<_Agent, _AnimationGraph>>()
			.expect("no animation data");
		assert!(graphs.get(&animation_data.graph).is_some());
	}

	#[test]
	fn store_animations_and_masks() {
		#[derive(Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<AnimationAsset, AnimationMask> {
				HashMap::from([
					(AnimationAsset::from("path/a"), 1),
					(AnimationAsset::from("path/b"), 2),
					(AnimationAsset::from("path/c"), 4),
				])
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.withf(|paths| {
					assert_eq!(
						HashSet::from([
							&AnimationAsset::from("path/a"),
							&AnimationAsset::from("path/b"),
							&AnimationAsset::from("path/c"),
						]),
						HashSet::from_iter(paths)
					);
					true
				})
				.return_const((
					_AnimationGraph,
					HashMap::from([
						(
							AnimationAsset::from("path/a"),
							Animations::Single(AnimationNodeIndex::new(1)),
						),
						(
							AnimationAsset::from("path/b"),
							Animations::Single(AnimationNodeIndex::new(2)),
						),
						(
							AnimationAsset::from("path/c"),
							Animations::Single(AnimationNodeIndex::new(3)),
						),
					]),
				));
		}));

		app.update();

		let animation_data = app
			.world()
			.get_resource::<AnimationData<_Agent, _AnimationGraph>>()
			.expect("no animation data");
		assert_eq!(
			HashMap::from([
				(
					AnimationAsset::from("path/a"),
					(Animations::Single(AnimationNodeIndex::new(1)), 1)
				),
				(
					AnimationAsset::from("path/b"),
					(Animations::Single(AnimationNodeIndex::new(2)), 2)
				),
				(
					AnimationAsset::from("path/c"),
					(Animations::Single(AnimationNodeIndex::new(3)), 4)
				),
			]),
			animation_data.animations
		);
	}
}
