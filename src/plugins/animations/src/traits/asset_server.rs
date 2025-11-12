pub(crate) mod animation_graph;

use super::LoadAnimationAssets;
use crate::components::animation_lookup::{AnimationClips, DirectionalIndices};
use bevy::prelude::*;
use common::{
	tools::path::Path,
	traits::{
		animation::{AnimationPath, Directional},
		load_asset::LoadAsset,
	},
};
use std::collections::HashMap;

impl<T, TGraph> LoadAnimationAssets<TGraph, AnimationClips> for T
where
	T: LoadAsset,
	TGraph: AnimationGraphTrait + Default,
{
	fn load_animation_assets(
		&mut self,
		animations: Vec<AnimationPath>,
	) -> (TGraph, HashMap<AnimationPath, AnimationClips>) {
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
) -> impl FnMut(AnimationPath) -> (AnimationPath, AnimationClips) + 'a
where
	TServer: LoadAsset,
	TGraph: AnimationGraphTrait,
{
	move |animation| {
		let animations = match &animation {
			AnimationPath::Single(path) => {
				let clip = server.load_asset(path);
				let index = graph.add_clip(clip, 1., blend_node);
				AnimationClips::Single(index)
			}
			AnimationPath::Directional(direction_paths) => {
				let blend_node = graph.add_blend(1., blend_node);
				let mut animations = DirectionalIndices::default();

				for (animation, path) in iter_parallel(&mut animations, direction_paths) {
					let clip = server.load_asset(path);
					*animation = graph.add_clip(clip, 0., blend_node);
				}

				AnimationClips::Directional(animations)
			}
		};
		(animation, animations)
	}
}

fn iter_parallel<'a>(
	dst: &'a mut DirectionalIndices,
	src: &'a Directional,
) -> [(&'a mut AnimationNodeIndex, &'a Path); 4] {
	[
		(&mut dst.forward, &src.forward),
		(&mut dst.backward, &src.backward),
		(&mut dst.left, &src.left),
		(&mut dst.right, &src.right),
	]
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
	use common::traits::load_asset::mock::MockAssetServer;
	use mockall::{mock, predicate::eq};
	use testing::new_handle;

	macro_rules! setup_graph {
		($setup:expr) => {
			type _AnimAssets = (_Graph, HashMap<AnimationPath, AnimationClips>);

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
		let mut server = MockAssetServer::default();

		let _: _AnimAssets = server.load_animation_assets(vec![]);
	}

	mod single_animation_asset {
		use super::*;
		use std::sync::LazyLock;

		#[test]
		fn load_asset() {
			setup_graph!(|_| {});
			let path = "a";
			let mut server = MockAssetServer::default()
				.path(path)
				.returns(new_handle::<AnimationClip>());

			let _: _AnimAssets = server.load_animation_assets(vec![AnimationPath::from(path)]);

			assert_eq!(1, server.calls(path))
		}

		#[test]
		fn add_loaded_clip_to_graph() {
			static HANDLE: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_additive_blend()
					.return_const(AnimationNodeIndex::new(11));
				mock.expect_add_clip()
					.times(1)
					.with(eq(HANDLE.clone()), eq(1.), eq(AnimationNodeIndex::new(11)))
					.return_const(AnimationNodeIndex::default());
			});
			let mut server = MockAssetServer::default().path("a").returns(HANDLE.clone());

			let _: _AnimAssets = server.load_animation_assets(vec![AnimationPath::from("a")]);
		}

		#[test]
		fn return_animation_node_index() {
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.return_const(AnimationNodeIndex::new(111));
			});
			let mut server = MockAssetServer::default();

			let (_, map): _AnimAssets =
				server.load_animation_assets(vec![AnimationPath::from("a")]);

			assert_eq!(
				HashMap::from([(
					AnimationPath::from("a"),
					AnimationClips::Single(AnimationNodeIndex::new(111))
				)]),
				map
			);
		}
	}

	mod directional_animation_asset {
		use super::*;
		use std::sync::LazyLock;

		#[test]
		fn load_assets() {
			setup_graph!(|_| {});
			let forward = "forward";
			let backward = "backward";
			let left = "left";
			let right = "right";
			let mut server = MockAssetServer::default();

			let _: _AnimAssets =
				server.load_animation_assets(vec![AnimationPath::Directional(Directional {
					forward: Path::from(forward),
					backward: Path::from(backward),
					left: Path::from(left),
					right: Path::from(right),
				})]);

			assert_eq!(
				(1, 1, 1, 1),
				(
					server.calls(forward),
					server.calls(backward),
					server.calls(left),
					server.calls(right)
				)
			)
		}

		#[test]
		fn add_loaded_clips_to_graph() {
			static FORWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static BACKWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static LEFT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static RIGHT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
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
						eq(FORWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(
						eq(BACKWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::new(4242)),
					)
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(LEFT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
				mock.expect_add_clip()
					.times(1)
					.with(eq(RIGHT.clone()), eq(0.), eq(AnimationNodeIndex::new(4242)))
					.return_const(AnimationNodeIndex::default());
			});
			let forward = "forward";
			let backward = "backward";
			let left = "left";
			let right = "right";
			let mut server = MockAssetServer::default()
				.path(forward)
				.returns(FORWARD.clone())
				.path(backward)
				.returns(BACKWARD.clone())
				.path(left)
				.returns(LEFT.clone())
				.path(right)
				.returns(RIGHT.clone());

			let _: _AnimAssets =
				server.load_animation_assets(vec![AnimationPath::Directional(Directional {
					forward: Path::from(forward),
					backward: Path::from(backward),
					left: Path::from(left),
					right: Path::from(right),
				})]);
		}

		#[test]
		fn return_animation_node_indices() {
			static FORWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static BACKWARD: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static LEFT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			static RIGHT: LazyLock<Handle<AnimationClip>> = LazyLock::new(new_handle);
			setup_graph!(|mock| {
				mock.expect_add_clip()
					.with(
						eq(FORWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(1));
				mock.expect_add_clip()
					.with(
						eq(BACKWARD.clone()),
						eq(0.),
						eq(AnimationNodeIndex::default()),
					)
					.return_const(AnimationNodeIndex::new(2));
				mock.expect_add_clip()
					.with(eq(LEFT.clone()), eq(0.), eq(AnimationNodeIndex::default()))
					.return_const(AnimationNodeIndex::new(3));
				mock.expect_add_clip()
					.with(eq(RIGHT.clone()), eq(0.), eq(AnimationNodeIndex::default()))
					.return_const(AnimationNodeIndex::new(4));
			});
			let forward = "forward";
			let backward = "backward";
			let left = "left";
			let right = "right";
			let mut server = MockAssetServer::default()
				.path(forward)
				.returns(FORWARD.clone())
				.path(backward)
				.returns(BACKWARD.clone())
				.path(left)
				.returns(LEFT.clone())
				.path(right)
				.returns(RIGHT.clone());
			let asset = AnimationPath::Directional(Directional {
				forward: Path::from(forward),
				backward: Path::from(backward),
				left: Path::from(left),
				right: Path::from(right),
			});

			let (_, map): _AnimAssets = server.load_animation_assets(vec![asset.clone()]);

			assert_eq!(
				HashMap::from([(
					asset,
					AnimationClips::Directional(DirectionalIndices {
						forward: AnimationNodeIndex::new(1),
						backward: AnimationNodeIndex::new(2),
						left: AnimationNodeIndex::new(3),
						right: AnimationNodeIndex::new(4)
					})
				)]),
				map
			);
		}
	}
}
