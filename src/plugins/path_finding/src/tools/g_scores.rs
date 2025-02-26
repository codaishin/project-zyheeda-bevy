use super::nav_grid_node::NavGridNode;
use std::collections::HashMap;

pub(crate) struct GScores(HashMap<NavGridNode, f32>);

impl GScores {
	pub(crate) fn new(start: NavGridNode) -> Self {
		Self(HashMap::from([(start, 0.)]))
	}

	pub(crate) fn insert(&mut self, node: NavGridNode, score: f32) {
		self.0.insert(node, score);
	}

	pub(crate) fn get(&self, node: &NavGridNode) -> f32 {
		self.0.get(node).cloned().unwrap_or(f32::INFINITY)
	}
}
