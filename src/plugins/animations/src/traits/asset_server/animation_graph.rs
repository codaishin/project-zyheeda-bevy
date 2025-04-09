use super::AnimationGraphTrait;
use bevy::prelude::*;

impl AnimationGraphTrait for AnimationGraph {
	fn add_clip(
		&mut self,
		clip: Handle<AnimationClip>,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex {
		self.add_clip(clip, weight, parent)
	}

	fn add_blend(&mut self, weight: f32, parent: AnimationNodeIndex) -> AnimationNodeIndex {
		self.add_blend(weight, parent)
	}

	fn add_additive_blend(
		&mut self,
		weight: f32,
		parent: AnimationNodeIndex,
	) -> AnimationNodeIndex {
		self.add_additive_blend(weight, parent)
	}

	fn root(&self) -> AnimationNodeIndex {
		self.root
	}
}

pub(crate) trait GetNodeMut {
	fn get_node_mut(&mut self, animation: AnimationNodeIndex) -> Option<&mut AnimationGraphNode>;
}

impl GetNodeMut for AnimationGraph {
	fn get_node_mut(&mut self, animation: AnimationNodeIndex) -> Option<&mut AnimationGraphNode> {
		self.get_mut(animation)
	}
}
