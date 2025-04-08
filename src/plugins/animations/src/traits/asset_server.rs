mod animation_graph;

use super::LoadAnimationAssets;
use bevy::prelude::*;
use common::traits::{
	animation::{AnimationAsset, MovementDirection},
	iteration::IterFinite,
	load_asset::LoadAsset,
};
use std::collections::HashMap;

impl<T, TGraph> LoadAnimationAssets<TGraph, AnimationNodeIndex> for T
where
	T: LoadAsset,
	TGraph: AnimationGraphTrait + Default,
{
	fn load_animation_assets(
		&mut self,
		animations: Vec<AnimationAsset>,
	) -> (TGraph, HashMap<AnimationAsset, Vec<AnimationNodeIndex>>) {
		let mut graph = TGraph::default();
		let blend_node = graph.add_additive_blend(1., graph.root());
		let load_clip = load_clip(self, &mut graph, blend_node);
		let animations = animations.into_iter().map(load_clip).collect();

		(graph, animations)
	}
}

fn load_clip<'a, TServer, TGraph>(
	server: &'a mut TServer,
	graph: &'a mut TGraph,
	blend_node: AnimationNodeIndex,
) -> impl FnMut(AnimationAsset) -> (AnimationAsset, Vec<AnimationNodeIndex>) + 'a
where
	TServer: LoadAsset,
	TGraph: AnimationGraphTrait,
{
	move |animation| {
		let index = match &animation {
			AnimationAsset::Single(path) => {
				let clip = server.load_asset(path.clone());
				let index = graph.add_clip(clip, 1., blend_node);
				vec![index]
			}
			AnimationAsset::Directional(direction_paths) => {
				let blend_node = graph.add_blend(1., blend_node);
				let mut weight = 1.;
				let mut indices = vec![];
				for direction in MovementDirection::iterator() {
					let clip = server.load_asset(direction_paths[direction].clone());
					let index = graph.add_clip(clip, weight, blend_node);
					indices.push(index);
					weight = 0.;
				}

				indices
			}
		};
		(animation, index)
	}
}

trait AnimationGraphTrait {
	fn add_clip(
		&mut self,
		clip: Handle<AnimationClip>,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex;
	fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
	fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex)
	-> AnimationNodeIndex;
	fn root(&self) -> AnimationNodeIndex;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetPath;
	use common::{
		simple_init,
		test_tools::utils::{new_handle, new_handle_from},
		traits::{load_asset::Path, mock::Mock},
	};
	use mockall::{mock, predicate::eq};
	use uuid::{Uuid, uuid};

	mock! {
		_AssetServer {}
		impl LoadAsset for _AssetServer {
			fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
			where
				TAsset: Asset,
				TPath: Into<AssetPath<'static>> + 'static;
		}
	}

	simple_init!(Mock_AssetServer);

	macro_rules! setup_graph {
		($setup:expr) => {
			type _AnimAssets = (_Graph, HashMap<AnimationAsset, Vec<AnimationNodeIndex>>);

			struct _Graph {
				mock: Mock_Graph,
			}

			// used to allow rust to self infer the setup argument type
			fn setup(func: impl Fn(&mut Mock_Graph)) -> impl Fn(&mut Mock_Graph) {
				func
			}

			impl Default for _Graph {
				fn default() -> Self {
					let mut mock = Mock_Graph::default();

					let setup = setup($setup);
					setup(&mut mock);

					// set defaults for everything not set up in setup
					mock.expect_add_clip()
						.return_const(AnimationNodeIndex::default());
					mock.expect_add_blend()
						.return_const(AnimationNodeIndex::default());
					mock.expect_add_additive_blend()
						.return_const(AnimationNodeIndex::default());
					mock.expect_root()
						.return_const(AnimationNodeIndex::default());
					Self { mock }
				}
			}

			impl AnimationGraphTrait for _Graph {
				fn add_clip(&mut self, clip: Handle<AnimationClip>, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_clip(clip, weight, parent)
				}

				fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_blend(weight, parent)
				}

				fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
					self.mock.add_additive_blend(weight, parent)
				}

				fn root(&self) -> AnimationNodeIndex {
					self.mock.root()
				}
			}

			mock! {
				_Graph {}
				impl AnimationGraphTrait for _Graph {
					fn add_clip(&mut self, clip: Handle<AnimationClip>, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn add_additive_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex;
					fn root(&self) -> AnimationNodeIndex;
				}
			}

		}
	}

	#[test]
	fn build_top_blend_node_for_mask_setup() {
		setup_graph!(|mock| {
			mock.expect_root().return_const(AnimationNodeIndex::new(42));
			mock.expect_add_additive_blend()
				.times(1)
				.with(eq(1.), eq(AnimationNodeIndex::new(42)))
				.return_const(AnimationNodeIndex::default());
		});
		let mut server = Mock_AssetServer::new_mock(|mock| {
			mock.expect_load_asset::<AnimationClip, Path>()
				.return_const(new_handle());
		});

		let _: _AnimAssets = server.load_animation_assets(vec![]);
	}

	mod single_animation_asset {
		use super::*;

		#[test]
		fn load_asset() {
			setup_graph!(|_| {});
			let path = Path::from("a");
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.times(1)
					.with(eq(path.clone()))
					.return_const(new_handle());
			});

			let _: _AnimAssets = server.load_animation_assets(vec![AnimationAsset::Single(path)]);
		}

		#[test]
		fn add_loaded_clip_to_graph() {
			const ASSET_ID: Uuid = uuid!("37f757ff-7e4a-4ac0-8ec3-4f18b5e446ec");
			setup_graph!(|mock| {
				mock.expect_add_additive_blend()
					.return_const(AnimationNodeIndex::new(11));
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(new_handle_from(ASSET_ID)),
						eq(1.),
						eq(AnimationNodeIndex::new(11)),
					)
					.return_const(AnimationNodeIndex::default());
			});
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.return_const(new_handle_from(ASSET_ID));
			});

			let _: _AnimAssets = server.load_animation_assets(vec![AnimationAsset::from("a")]);
		}

		#[test]
		fn return_animation_node_index() {
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.return_const(AnimationNodeIndex::new(111));
			});
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.return_const(new_handle());
			});

			let (_, map): _AnimAssets =
				server.load_animation_assets(vec![AnimationAsset::from("a")]);

			assert_eq!(
				HashMap::from([(
					AnimationAsset::from("a"),
					vec![AnimationNodeIndex::new(111)]
				)]),
				map
			);
		}
	}

	mod directional_animation_asset {
		use super::*;
		use common::traits::animation::Directions;

		#[test]
		fn load_assets() {
			setup_graph!(|_| {});
			let forward = Path::from("forward");
			let backward = Path::from("backward");
			let left = Path::from("left");
			let right = Path::from("right");
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.times(1)
					.with(eq(forward.clone()))
					.return_const(new_handle());
				mock.expect_load_asset::<AnimationClip, Path>()
					.times(1)
					.with(eq(backward.clone()))
					.return_const(new_handle());
				mock.expect_load_asset::<AnimationClip, Path>()
					.times(1)
					.with(eq(left.clone()))
					.return_const(new_handle());
				mock.expect_load_asset::<AnimationClip, Path>()
					.times(1)
					.with(eq(right.clone()))
					.return_const(new_handle());
			});

			let _: _AnimAssets =
				server.load_animation_assets(vec![AnimationAsset::from(Directions {
					forward,
					backward,
					left,
					right,
				})]);
		}

		#[test]
		fn add_loaded_clips_to_graph() {
			const FORWARD_ID: Uuid = uuid!("3786ced0-3930-472c-aa84-8b0338cce8c8");
			const BACKWARD_ID: Uuid = uuid!("c4d79d59-49b2-45aa-bc35-36ddad029f28");
			const LEFT_ID: Uuid = uuid!("f6b9a572-6a8a-4846-b88e-5e936f4492d5");
			const RIGHT_ID: Uuid = uuid!("80176d8a-23c4-4dc0-a6c7-2b6cf5e23621");
			setup_graph!(|mock| {
				mock.expect_add_additive_blend()
					.return_const(AnimationNodeIndex::new(11));
				mock.expect_add_blend()
					.times(1)
					.with(eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::new(4242));
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(new_handle_from(FORWARD_ID)),
						eq(1.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(new_handle_from(BACKWARD_ID)),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(new_handle_from(LEFT_ID)),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(new_handle_from(RIGHT_ID)),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
			});
			let forward = Path::from("forward");
			let backward = Path::from("backward");
			let left = Path::from("left");
			let right = Path::from("right");
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(forward.clone()))
					.return_const(new_handle_from(FORWARD_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(backward.clone()))
					.return_const(new_handle_from(BACKWARD_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(left.clone()))
					.return_const(new_handle_from(LEFT_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(right.clone()))
					.return_const(new_handle_from(RIGHT_ID));
			});

			let _: _AnimAssets =
				server.load_animation_assets(vec![AnimationAsset::from(Directions {
					forward,
					backward,
					left,
					right,
				})]);
		}

		#[test]
		fn return_animation_node_indices() {
			const FORWARD_ID: Uuid = uuid!("3786ced0-3930-472c-aa84-8b0338cce8c8");
			const BACKWARD_ID: Uuid = uuid!("c4d79d59-49b2-45aa-bc35-36ddad029f28");
			const LEFT_ID: Uuid = uuid!("f6b9a572-6a8a-4846-b88e-5e936f4492d5");
			const RIGHT_ID: Uuid = uuid!("80176d8a-23c4-4dc0-a6c7-2b6cf5e23621");
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.with(
						eq(new_handle_from(FORWARD_ID)),
						eq(1.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(1));
				mock.expect_add_clip()
					.with(
						eq(new_handle_from(BACKWARD_ID)),
						eq(0.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(2));
				mock.expect_add_clip()
					.with(
						eq(new_handle_from(LEFT_ID)),
						eq(0.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(3));
				mock.expect_add_clip()
					.with(
						eq(new_handle_from(RIGHT_ID)),
						eq(0.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(4));
			});
			let forward = Path::from("forward");
			let backward = Path::from("backward");
			let left = Path::from("left");
			let right = Path::from("right");
			let mut server = Mock_AssetServer::new_mock(|mock| {
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(forward.clone()))
					.return_const(new_handle_from(FORWARD_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(backward.clone()))
					.return_const(new_handle_from(BACKWARD_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(left.clone()))
					.return_const(new_handle_from(LEFT_ID));
				mock.expect_load_asset::<AnimationClip, Path>()
					.with(eq(right.clone()))
					.return_const(new_handle_from(RIGHT_ID));
			});
			let asset = AnimationAsset::from(Directions {
				forward,
				backward,
				left,
				right,
			});

			let (_, map): _AnimAssets = server.load_animation_assets(vec![asset.clone()]);

			assert_eq!(
				HashMap::from([(
					asset,
					vec![
						AnimationNodeIndex::new(1),
						AnimationNodeIndex::new(2),
						AnimationNodeIndex::new(3),
						AnimationNodeIndex::new(4)
					]
				)]),
				map
			);
		}
	}
}
