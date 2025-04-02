use crate::{resource::AnimationData, traits::LoadAnimationAssets};
use bevy::prelude::*;
use common::{
	resources::Shared,
	traits::{
		animation::{AnimationMaskRoot, GetAnimationDefinitions},
		load_asset::Path,
	},
};

impl<TAgent> InitAnimationClips for TAgent
where
	TAgent: GetAnimationDefinitions + Sync + Send + 'static,
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskRoot: From<&'a Self::TAnimationMask>,
{
}

pub(crate) trait InitAnimationClips
where
	Self: GetAnimationDefinitions + Sync + Send + Sized + 'static,
	for<'a> AnimationMask: From<&'a Self::TAnimationMask>,
	for<'a> AnimationMaskRoot: From<&'a Self::TAnimationMask>,
{
	fn init_animation_clips<
		TAnimationGraph: Asset + Sync + Send + 'static,
		TServer: Resource + LoadAnimationAssets<TAnimationGraph, AnimationNodeIndex>,
	>(
		mut commands: Commands,
		server: Res<TServer>,
		mut graphs: ResMut<Assets<TAnimationGraph>>,
	) {
		let paths = Self::animation_definitions()
			.into_iter()
			.map(to_animation_mask)
			.collect::<Vec<_>>();
		let (graph, indices) = server.load_animation_assets(&paths);
		let graph = graphs.add(graph);

		commands.insert_resource(Shared::from(indices));
		commands.insert_resource(AnimationData::<Self, TAnimationGraph>::new(graph));
	}
}

fn to_animation_mask<TAnimationMask>(
	(mask, path): (Option<TAnimationMask>, Path),
) -> (AnimationMask, Path)
where
	for<'a> AnimationMask: From<&'a TAnimationMask>,
{
	let mask = mask
		.map(|mask| AnimationMask::from(&mask))
		.unwrap_or(AnimationMask::MAX);

	(mask, path)
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
			animation::AnimationMaskRoot,
			load_asset::Path,
			nested_mock::NestedMocks,
			thread_safe::ThreadSafe,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Resource, NestedMocks)]
	struct _Server {
		mock: Mock_Server,
	}

	#[derive(Debug, PartialEq, Clone, TypePath, Asset)]
	struct _AnimationGraph;

	struct _Mask(AnimationMask);

	impl From<&_Mask> for AnimationMask {
		fn from(_Mask(mask): &_Mask) -> Self {
			*mask
		}
	}

	impl From<&_Mask> for AnimationMaskRoot {
		fn from(_: &_Mask) -> Self {
			panic!("SHOULD NOT BE USED HERE")
		}
	}

	#[automock]
	impl LoadAnimationAssets<_AnimationGraph, AnimationNodeIndex> for _Server {
		fn load_animation_assets(
			&self,
			animations: &[(AnimationMask, Path)],
		) -> (_AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
			self.mock.load_animation_assets(animations)
		}
	}

	fn setup<TAgent>(server: _Server) -> App
	where
		TAgent: GetAnimationDefinitions + ThreadSafe,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskRoot: From<&'a TAgent::TAnimationMask>,
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

		const MASK_1: AnimationMask = 0b0001;
		const MASK_2: AnimationMask = 0b0010;

		impl GetAnimationDefinitions for _Agent {
			type TAnimationMask = _Mask;

			fn animation_definitions() -> Vec<(Option<_Mask>, Path)> {
				vec![
					(Some(_Mask(MASK_1)), Path::from("path/a")),
					(Some(_Mask(MASK_2)), Path::from("path/b")),
					(None, Path::from("path/c")),
				]
			}
		}

		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.with(eq(vec![
					(MASK_1, Path::from("path/a")),
					(MASK_2, Path::from("path/b")),
					(AnimationMask::MAX, Path::from("path/c")),
				]))
				.return_const((
					_AnimationGraph,
					HashMap::from([
						(Path::from("path/a"), AnimationNodeIndex::new(1)),
						(Path::from("path/a"), AnimationNodeIndex::new(2)),
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
				(Path::from("path/a"), AnimationNodeIndex::new(2)),
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

			fn animation_definitions() -> Vec<(Option<_Mask>, Path)> {
				vec![]
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
