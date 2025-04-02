use super::LoadAnimationAssets;
use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::collections::HashMap;

impl LoadAnimationAssets<AnimationGraph, AnimationNodeIndex> for AssetServer {
	fn load_animation_assets(
		&self,
		animations: &[(AnimationMask, Path)],
	) -> (AnimationGraph, HashMap<Path, AnimationNodeIndex>) {
		let mut graph = AnimationGraph::default();
		let blend_node = graph.add_additive_blend(1., graph.root);
		let load_clip = load_clip(self, &mut graph, blend_node);
		let animations = animations.iter().map(load_clip).collect();

		(graph, animations)
	}
}

fn load_clip<'a>(
	server: &'a AssetServer,
	graph: &'a mut AnimationGraph,
	blend_node: AnimationNodeIndex,
) -> impl FnMut(&(AnimationMask, Path)) -> (Path, AnimationNodeIndex) + 'a {
	move |(mask, path)| {
		let clip = server.load(path.clone());
		let index = graph.add_clip_with_mask(clip, invert(mask), 1., blend_node);
		(path.clone(), index)
	}
}

/// Inverting, because the mask we need actually defines what mask ids this animation
/// does not affect.
fn invert(mask: &AnimationMask) -> AnimationMask {
	!*mask
}
