use crate::{resource::AnimationData, traits::LoadAnimationAssets};
use bevy::{
	asset::{Asset, Assets},
	prelude::{AnimationNodeIndex, Commands, Res, ResMut, Resource},
};
use common::{resources::Shared, traits::animation::GetAnimationPaths};

impl<TAgent> InitAnimationClips for TAgent where TAgent: GetAnimationPaths + Sync + Send + 'static {}

pub(crate) trait InitAnimationClips
where
	Self: GetAnimationPaths + Sync + Send + Sized + 'static,
{
	fn init_animation_clips<
		TAnimationGraph: Asset + Sync + Send + 'static,
		TServer: Resource + LoadAnimationAssets<TAnimationGraph, AnimationNodeIndex>,
	>(
		mut commands: Commands,
		server: Res<TServer>,
		mut graphs: ResMut<Assets<TAnimationGraph>>,
	) {
		let paths = Self::animation_paths();
		let (graph, indices) = server.load_animation_assets(&paths);
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
		traits::{load_asset::Path, nested_mock::NestedMocks},
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

	#[automock]
	impl LoadAnimationAssets<_AnimationGraph, AnimationNodeIndex> for _Server {
		fn load_animation_assets(
			&self,
			paths: &[Path],
		) -> (_AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
			self.mock.load_animation_assets(paths)
		}
	}

	fn setup<TAgent: GetAnimationPaths + Sync + Send + 'static>(server: _Server) -> App {
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

		impl GetAnimationPaths for _Agent {
			fn animation_paths() -> Vec<Path> {
				vec![Path::from("path/a"), Path::from("path/b")]
			}
		}
		let mut app = setup::<_Agent>(_Server::new().with_mock(|mock| {
			mock.expect_load_animation_assets()
				.with(eq(vec![Path::from("path/a"), Path::from("path/b")]))
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

		impl GetAnimationPaths for _Agent {
			fn animation_paths() -> Vec<Path> {
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
