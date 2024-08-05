use crate::{
	resource::AnimationData,
	traits::{GetAnimationPaths, LoadAnimationAssets},
};
use bevy::{
	asset::{Asset, Assets},
	prelude::{Commands, Res, ResMut, Resource},
};
use common::resources::Shared;

pub(crate) fn init_animation_clips<
	TAgent: GetAnimationPaths + Sync + Send + 'static,
	TAnimationGraph: Asset + Sync + Send + 'static,
	TAnimationNode: Clone + Sync + Send + 'static,
	TServer: Resource + LoadAnimationAssets<TAnimationGraph, TAnimationNode>,
>(
	mut commands: Commands,
	server: Res<TServer>,
	mut graphs: ResMut<Assets<TAnimationGraph>>,
) {
	let paths = TAgent::animation_paths();
	let (graph, indices) = server.load_animation_assets(&paths);
	let graph = graphs.add(graph);

	commands.insert_resource(Shared::from(indices));
	commands.insert_resource(AnimationData::<TAgent, TAnimationGraph>::new(graph));
}

#[cfg(test)]
mod tests {
	use crate::resource::AnimationData;

	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::Asset,
		reflect::TypePath,
	};
	use common::{
		resources::Shared,
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMock},
	};
	use macros::NestedMock;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Resource, NestedMock)]
	struct _Server {
		mock: Mock_Server,
	}

	#[derive(Debug, PartialEq, Clone, TypePath, Asset)]
	struct _AnimationGraph;

	#[derive(Debug, PartialEq, Clone)]
	struct _AnimationIndex(&'static str);

	#[automock]
	impl LoadAnimationAssets<_AnimationGraph, _AnimationIndex> for _Server {
		fn load_animation_assets(
			&self,
			paths: &[Path],
		) -> (_AnimationGraph, HashMap<Path, _AnimationIndex>) {
			self.mock.load_animation_assets(paths)
		}
	}

	fn setup<TAgent: GetAnimationPaths + Sync + Send + 'static>(server: _Server) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(server);
		app.insert_resource(Assets::<_AnimationGraph>::default());
		app.add_systems(
			Update,
			init_animation_clips::<TAgent, _AnimationGraph, _AnimationIndex, _Server>,
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
		let mut app = setup::<_Agent>(_Server::new_mock(|mock| {
			mock.expect_load_animation_assets()
				.with(eq(vec![Path::from("path/a"), Path::from("path/b")]))
				.return_const((
					_AnimationGraph,
					HashMap::from([
						(Path::from("path/a"), _AnimationIndex("path/a")),
						(Path::from("path/a"), _AnimationIndex("path/b")),
					]),
				));
		}));

		app.update();

		let indices = app.world().get_resource::<Shared<Path, _AnimationIndex>>();

		assert_eq!(
			Some(&Shared::new([
				(Path::from("path/a"), _AnimationIndex("path/a")),
				(Path::from("path/a"), _AnimationIndex("path/b")),
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

		let mut app = setup::<_Agent>(_Server::new_mock(|mock| {
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
