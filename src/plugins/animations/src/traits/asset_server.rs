use super::LoadAnimationAssets;
use bevy::{
	asset::AssetServer,
	prelude::{AnimationGraph, AnimationNodeIndex},
};
use common::traits::load_asset::Path;
use std::collections::HashMap;

impl LoadAnimationAssets<AnimationGraph, AnimationNodeIndex> for AssetServer {
	fn load_animation_assets(
		&self,
		paths: &[Path],
	) -> (AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
		let mut graph = AnimationGraph::default();
		let load_clip = load_clip(self, &mut graph);
		let animations = paths.iter().map(load_clip).collect();

		(graph, animations)
	}
}

fn load_clip<'a>(
	server: &'a AssetServer,
	graph: &'a mut AnimationGraph,
) -> impl FnMut(&Path) -> (Path, AnimationNodeIndex) + 'a {
	|path| {
		let clip = server.load(path.clone());
		let index = graph.add_clip(clip, 1., graph.root);
		(path.clone(), index)
	}
}
