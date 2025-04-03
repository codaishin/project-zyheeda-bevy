use crate::{resource::AnimationData, traits::LoadAnimationAssets};
use bevy::prelude::*;
use common::{
	resources::Shared,
	traits::animation::{AnimationMaskDefinition, GetAnimationDefinitions},
};

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
		TServer: Resource + LoadAnimationAssets<TAnimationGraph, AnimationNodeIndex>,
	>(
		mut commands: Commands,
		server: Res<TServer>,
		mut graphs: ResMut<Assets<TAnimationGraph>>,
	) {
		let animations = Self::animations().keys().cloned().collect::<Vec<_>>();
		let (graph, indices) = server.load_animation_assets(animations);
		let graph = graphs.add(graph);

		commands.insert_resource(Shared::from(indices));
		commands.insert_resource(AnimationData::<Self, TAnimationGraph>::new(graph));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::resource::AnimationData;
	use bevy::{
		app::{App, Update},
		prelude::Asset,
		reflect::TypePath,
	};
	use common::{
		resources::Shared,
		test_tools::utils::SingleThreadedApp,
		traits::{
			animation::AnimationMaskDefinition,
			iteration::{Iter, IterFinite},
			load_asset::Path,
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
	impl LoadAnimationAssets<_AnimationGraph, AnimationNodeIndex> for _Server {
		fn load_animation_assets(
			&self,
			animations: Vec<Path>,
		) -> (_AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
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
	fn store_animation_clips() {
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<Path, AnimationMask> {
				HashMap::from([
					(Path::from("path/a"), AnimationMask::default()),
					(Path::from("path/b"), AnimationMask::default()),
					(Path::from("path/c"), AnimationMask::default()),
				])
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.withf(|paths| {
					assert_eq!(
						HashSet::from([
							&Path::from("path/a"),
							&Path::from("path/b"),
							&Path::from("path/c"),
						]),
						HashSet::from_iter(paths)
					);
					true
				})
				.return_const((
					_AnimationGraph,
					HashMap::from([
						(Path::from("path/a"), AnimationNodeIndex::new(1)),
						(Path::from("path/b"), AnimationNodeIndex::new(2)),
						(Path::from("path/c"), AnimationNodeIndex::new(3)),
					]),
				));
		}));

		app.update();

		let indices = app
			.world()
			.get_resource::<Shared<Path, AnimationNodeIndex>>();

		assert_eq!(
			Some(&Shared::new([
				(Path::from("path/a"), AnimationNodeIndex::new(1)),
				(Path::from("path/b"), AnimationNodeIndex::new(2)),
				(Path::from("path/c"), AnimationNodeIndex::new(3)),
			])),
			indices
		)
	}

	#[test]
	fn store_animation_graph() {
		#[derive(Debug, PartialEq)]
		struct _Agent;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animations() -> HashMap<Path, AnimationMask> {
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
}
