use crate::{
	components::nav_grid::NavGridData,
	tools::{
		closed_list::{walk_without_redundant::WithoutRedundantNodes, ClosedList},
		g_scores::GScores,
		line_wide::LineWide,
		nav_grid_node::NavGridNode,
		open_list::OpenList,
	},
};
use bevy::prelude::*;
use common::traits::handles_path_finding::ComputePath;

pub struct ThetaStar {
	sqrt_2: f32,
	grid: NavGridData,
}

impl ThetaStar {
	const NEIGHBORS: &'static [(i32, i32)] = &[
		(-1, -1),
		(-1, 0),
		(-1, 1),
		(0, -1),
		(0, 1),
		(1, -1),
		(1, 0),
		(1, 1),
	];

	fn neighbors<'a>(&'a self, center: &'a NavGridNode) -> impl Iterator<Item = NavGridNode> + 'a {
		Self::NEIGHBORS
			.iter()
			.map(|(x, y)| NavGridNode {
				x: center.x + x,
				y: center.y + y,
			})
			.filter(|NavGridNode { x, y, .. }| {
				x <= &self.grid.max.x
					&& x >= &self.grid.min.x
					&& y <= &self.grid.max.y
					&& y >= &self.grid.min.y
			})
	}

	fn distance(&self, a: NavGridNode, b: NavGridNode) -> f32 {
		let d_x = a.x.abs_diff(b.x) as f32;
		let d_y = a.y.abs_diff(b.y) as f32;
		let (long, short) = match d_x > d_y {
			true => (d_x, d_y),
			false => (d_y, d_x),
		};
		self.sqrt_2 * short + (long - short)
	}

	fn los(&self, a: NavGridNode, b: NavGridNode) -> bool {
		LineWide::new(a, b).all(|n| !self.grid.obstacles.contains(&n))
	}

	fn vertex(
		&self,
		closed: &ClosedList,
		g_scores: &GScores,
		current: NavGridNode,
		neighbor: NavGridNode,
	) -> Option<(NavGridNode, f32)> {
		match closed.parent(&current) {
			Some(parent) if self.los(*parent, neighbor) => self.relax(g_scores, *parent, neighbor),
			_ if self.los(current, neighbor) => self.relax(g_scores, current, neighbor),
			_ => None,
		}
	}

	fn relax(
		&self,
		g_scores: &GScores,
		current: NavGridNode,
		neighbor: NavGridNode,
	) -> Option<(NavGridNode, f32)> {
		let g = g_scores.get(&current) + self.distance(current, neighbor);

		if g >= g_scores.get(&neighbor) {
			return None;
		}

		Some((current, g))
	}
}

impl Default for ThetaStar {
	fn default() -> Self {
		Self {
			sqrt_2: f32::sqrt(2.),
			grid: default(),
		}
	}
}

impl From<NavGridData> for ThetaStar {
	fn from(grid: NavGridData) -> Self {
		Self {
			sqrt_2: f32::sqrt(2.),
			grid,
		}
	}
}

impl ComputePath for ThetaStar {
	fn compute_path(&self, start: Vec3, end: Vec3) -> Vec<Vec3> {
		let start = NavGridNode::from(start);
		let end = NavGridNode::from(end);
		let dist_f = |a, b| self.distance(a, b);
		let los_f = |a, b| self.los(a, b);
		let mut open = OpenList::new(end, start, &dist_f);
		let mut closed = ClosedList::new(end);
		let mut g_scores = GScores::new(end);

		while let Some(current) = open.pop_lowest_f() {
			if current == start {
				return closed
					.walk_back_from(current)
					.without_redundant_nodes(los_f)
					.skip(1)
					.map(Vec3::from)
					.collect();
			}

			for neighbor in self.neighbors(&current) {
				if self.grid.obstacles.contains(&neighbor) {
					continue;
				}

				let Some((current, g)) = self.vertex(&closed, &g_scores, current, neighbor) else {
					continue;
				};

				open.push(neighbor, g);
				closed.insert(neighbor, current);
				g_scores.insert(neighbor, g);
			}
		}

		vec![]
	}
}
