use crate::tools::nav_grid_node::NavGridNode;

pub(crate) trait ComputePathLazy {
	fn compute_path(
		&self,
		start: NavGridNode,
		end: NavGridNode,
	) -> impl Iterator<Item = NavGridNode>;
}
