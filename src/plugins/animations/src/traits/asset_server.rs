use super::LoadAnimationAssets;
use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::collections::HashMap;

impl LoadAnimationAssets<AnimationGraph, AnimationNodeIndex> for AssetServer {
	fn load_animation_assets(
		&self,
		animations: Vec<Path>,
	) -> (AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
		let mut graph = AnimationGraph::default();
		let blend_node = graph.add_additive_blend(1., graph.root);
		let load_clip = load_clip(self, &mut graph, blend_node);
		let animations = animations.into_iter().map(load_clip).collect();

		(graph, animations)
	}
}

fn load_clip<'a>(
	server: &'a AssetServer,
	graph: &'a mut AnimationGraph,
	blend_node: AnimationNodeIndex,
) -> impl FnMut(Path) -> (Path, AnimationNodeIndex) + 'a {
	move |path| {
		let clip = server.load(path.clone());
		let index = graph.add_clip(clip, 1., blend_node);
		(path, index)
	}
}
