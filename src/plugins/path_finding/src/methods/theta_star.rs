use crate::{
	tools::{closed_list::ClosedList, g_scores::GScores, open_list::OpenList},
	traits::compute_path_lazy::ComputePathLazy,
};
use common::traits::handles_map_generation::{
	GraphLineOfSight,
	GraphObstacle,
	GraphSuccessors,
	GraphTranslation,
};
use std::hash::Hash;

pub struct ThetaStar {
	sqrt_2: f32,
}

impl ThetaStar {
	fn distance<TGraph>(&self, graph: &TGraph, a: &TGraph::TTNode, b: &TGraph::TTNode) -> f32
	where
		TGraph: GraphTranslation,
	{
		let a = graph.translation(a);
		let b = graph.translation(b);
		let d_x = (a.x - b.x).abs();
		let d_z = (a.z - b.z).abs();
		let (long, short) = match d_x > d_z {
			true => (d_x, d_z),
			false => (d_z, d_x),
		};
		self.sqrt_2 * short + (long - short)
	}

	fn vertex<TGraph>(
		&self,
		graph: &TGraph,
		closed: &ClosedList<TGraph::TLNode>,
		g_scores: &GScores<TGraph::TLNode>,
		current: &TGraph::TLNode,
		neighbor: &TGraph::TLNode,
	) -> Option<(TGraph::TLNode, f32)>
	where
		TGraph: GraphLineOfSight + GraphTranslation<TTNode = TGraph::TLNode>,
		TGraph::TLNode: Eq + Hash + Copy,
	{
		let los = |a, b| graph.line_of_sight(a, b);

		match closed.parent(current) {
			Some(parent) if los(parent, neighbor) => self.relax(graph, g_scores, parent, neighbor),
			_ if los(current, neighbor) => self.relax(graph, g_scores, current, neighbor),
			_ => None,
		}
	}

	fn relax<TGraph>(
		&self,
		graph: &TGraph,
		g_scores: &GScores<TGraph::TTNode>,
		current: &TGraph::TTNode,
		neighbor: &TGraph::TTNode,
	) -> Option<(TGraph::TTNode, f32)>
	where
		TGraph: GraphTranslation,
		TGraph::TTNode: Eq + Hash + Copy,
	{
		let g = g_scores.get(current) + self.distance(graph, current, neighbor);

		if g >= g_scores.get(neighbor) {
			return None;
		}

		Some((*current, g))
	}
}

impl Default for ThetaStar {
	fn default() -> Self {
		Self {
			sqrt_2: f32::sqrt(2.),
		}
	}
}

impl<TGraph> ComputePathLazy<TGraph> for ThetaStar
where
	TGraph::TSNode: Eq + Hash + Copy,
	TGraph: GraphSuccessors
		+ GraphLineOfSight<TLNode = TGraph::TSNode>
		+ GraphObstacle<TONode = TGraph::TSNode>
		+ GraphTranslation<TTNode = TGraph::TSNode>,
{
	fn compute_path(
		&self,
		graph: &TGraph,
		start: TGraph::TSNode,
		end: TGraph::TSNode,
	) -> impl Iterator<Item = TGraph::TSNode> {
		let mut open = OpenList::new(end, start, |a, b| self.distance(graph, a, b));
		let mut closed = ClosedList::new(end);
		let mut g_scores = GScores::new(end);

		while let Some(current) = open.pop_lowest_f() {
			if current == start {
				return IterPath::Some(closed.walk_back_from(current));
			}

			for neighbor in graph.successors(&current) {
				if graph.is_obstacle(&neighbor) {
					continue;
				}

				let current = self.vertex(graph, &closed, &g_scores, &current, &neighbor);

				let Some((current, g)) = current else {
					continue;
				};

				open.push(neighbor, g);
				closed.insert(neighbor, current);
				g_scores.insert(neighbor, g);
			}
		}

		IterPath::None
	}
}

enum IterPath<T> {
	Some(T),
	None,
}

impl<T> Iterator for IterPath<T>
where
	T: Iterator,
{
	type Item = T::Item;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			IterPath::Some(iter) => iter.next(),
			IterPath::None => None,
		}
	}
}
