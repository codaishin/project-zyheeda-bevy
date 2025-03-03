use std::{collections::HashMap, hash::Hash};

pub(crate) struct GScores<TNode>(HashMap<TNode, f32>)
where
	TNode: Eq + Hash;

impl<TNode> GScores<TNode>
where
	TNode: Eq + Hash,
{
	pub(crate) fn new(start: TNode) -> Self {
		Self(HashMap::from([(start, 0.)]))
	}

	pub(crate) fn insert(&mut self, node: TNode, score: f32) {
		self.0.insert(node, score);
	}

	pub(crate) fn get(&self, node: &TNode) -> f32 {
		self.0.get(node).cloned().unwrap_or(f32::INFINITY)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let start = "start";
		let scores = GScores::new(start);

		assert_eq!(0., scores.get(&start));
	}

	#[test]
	fn insert() {
		let node = "node";
		let mut scores = GScores::new("start");

		scores.insert(node, 42.);

		assert_eq!(42., scores.get(&node));
	}
}
