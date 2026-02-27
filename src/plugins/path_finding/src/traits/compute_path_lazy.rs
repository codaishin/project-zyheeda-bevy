use common::traits::handles_map_generation::{
	GraphGroundPosition,
	GraphLineOfSight,
	GraphObstacle,
	GraphSuccessors,
};
use std::hash::Hash;

pub trait ComputePathLazy<TGraph>
where
	TGraph::TSNode: Eq + Hash + Copy,
	TGraph: GraphSuccessors
		+ GraphLineOfSight<TLNode = TGraph::TSNode>
		+ GraphObstacle<TONode = TGraph::TSNode>
		+ GraphGroundPosition<TTNode = TGraph::TSNode>,
{
	fn compute_path(
		&self,
		graph: &TGraph,
		start: TGraph::TSNode,
		end: TGraph::TSNode,
	) -> impl Iterator<Item = TGraph::TSNode>;
}
