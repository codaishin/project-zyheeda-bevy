use super::{SquareGridGraph, context::SquareGridContext};
use crate::traits::{
	grid_min::GridMin,
	to_subdivided::{SubdivisionError, ToSubdivided},
};
use bevy::utils::default;

impl ToSubdivided for SquareGridGraph {
	fn to_subdivided(&self, subdivisions: u8) -> Result<Self, SubdivisionError> {
		match subdivisions {
			0 => Ok(self.clone()),
			_ => subdivide(self, subdivisions),
		}
	}
}

fn subdivide(
	source: &SquareGridGraph,
	subdivisions: u8,
) -> Result<SquareGridGraph, SubdivisionError> {
	let factor = subdivisions + 1;
	let source_grid = source.context;
	let mut subdivided: SquareGridGraph = SquareGridGraph {
		context: SquareGridContext {
			cell_count_x: source_grid
				.cell_count_x
				.multiply_by(factor)
				.map_err(SubdivisionError::CellCountMaxedOut)?,
			cell_count_z: source_grid
				.cell_count_z
				.multiply_by(factor)
				.map_err(SubdivisionError::CellCountMaxedOut)?,
			cell_distance: source_grid
				.cell_distance
				.divided_by(factor)
				.map_err(SubdivisionError::CellDistanceZero)?,
		},
		..default()
	};
	let min = subdivided.context.grid_min();
	let mut translation = min;

	for x in 0..*subdivided.context.cell_count_x {
		for z in 0..*subdivided.context.cell_count_z {
			let source_key = source_key(x, z, factor);
			if source.nodes.contains_key(&source_key) {
				subdivided.nodes.insert((x, z), translation);
			}
			if source.extra.obstacles.contains(&source_key) {
				subdivided.extra.obstacles.insert((x, z));
			}
			translation.z += *subdivided.context.cell_distance;
		}

		translation.x += *subdivided.context.cell_distance;
		translation.z = min.z;
	}

	Ok(subdivided)
}

fn source_key(x: u32, z: u32, factor: u8) -> (u32, u32) {
	let factor = factor as f32;

	((x as f32 / factor) as u32, (z as f32 / factor) as u32)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::square_grid_graph::{
		Obstacles,
		context::{CellCount, CellDistance, SquareGridContext},
	};
	use bevy::math::Vec3;
	use common::traits::handles_map_generation::GroundPosition;
	use macros::new_valid;
	use std::collections::{HashMap, HashSet};

	#[test]
	fn subdivide_0_returns_same_graph() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([
				((0, 0), GroundPosition(Vec3::new(-5., 0., -5.))),
				((0, 1), GroundPosition(Vec3::new(-5., 0., 5.))),
				((1, 0), GroundPosition(Vec3::new(5., 0., -5.))),
				((1, 1), GroundPosition(Vec3::new(5., 0., 5.))),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(0, 1), (1, 1)]),
			},
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 2),
				cell_count_z: new_valid!(CellCount, 2),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};

		let resized = graph.clone().to_subdivided(0);

		assert_eq!(Ok(graph), resized);
	}

	#[test]
	fn subdivide_1_with_all_empty_nodes() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 2),
				cell_count_z: new_valid!(CellCount, 2),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			Ok(SquareGridGraph {
				nodes: HashMap::from([]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: SquareGridContext {
					cell_count_x: new_valid!(CellCount, 4),
					cell_count_z: new_valid!(CellCount, 4),
					cell_distance: new_valid!(CellDistance, 5.),
				},
			}),
			resized
		);
	}

	#[test]
	fn subdivide_1_without_obstacles() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([
				((0, 0), GroundPosition(Vec3::new(-5., 0., -5.))),
				((0, 1), GroundPosition(Vec3::new(-5., 0., 5.))),
				((1, 0), GroundPosition(Vec3::new(5., 0., -5.))),
				((1, 1), GroundPosition(Vec3::new(5., 0., 5.))),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 2),
				cell_count_z: new_valid!(CellCount, 2),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};

		let resized = graph.to_subdivided(1);

		assert_eq!(
			Ok(SquareGridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), GroundPosition(Vec3::new(-7.5, 0., -7.5))),
					((0, 1), GroundPosition(Vec3::new(-7.5, 0., -2.5))),
					((1, 0), GroundPosition(Vec3::new(-2.5, 0., -7.5))),
					((1, 1), GroundPosition(Vec3::new(-2.5, 0., -2.5))),
					// old (1, 0)
					((2, 0), GroundPosition(Vec3::new(2.5, 0., -7.5))),
					((2, 1), GroundPosition(Vec3::new(2.5, 0., -2.5))),
					((3, 0), GroundPosition(Vec3::new(7.5, 0., -7.5))),
					((3, 1), GroundPosition(Vec3::new(7.5, 0., -2.5))),
					// old (0, 1)
					((0, 2), GroundPosition(Vec3::new(-7.5, 0., 2.5))),
					((0, 3), GroundPosition(Vec3::new(-7.5, 0., 7.5))),
					((1, 2), GroundPosition(Vec3::new(-2.5, 0., 2.5))),
					((1, 3), GroundPosition(Vec3::new(-2.5, 0., 7.5))),
					// old (1, 1)
					((2, 2), GroundPosition(Vec3::new(2.5, 0., 2.5))),
					((2, 3), GroundPosition(Vec3::new(2.5, 0., 7.5))),
					((3, 2), GroundPosition(Vec3::new(7.5, 0., 2.5))),
					((3, 3), GroundPosition(Vec3::new(7.5, 0., 7.5))),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: SquareGridContext {
					cell_count_x: new_valid!(CellCount, 4),
					cell_count_z: new_valid!(CellCount, 4),
					cell_distance: new_valid!(CellDistance, 5.),
				},
			}),
			resized
		);
	}

	#[test]
	fn subdivide_1_without_obstacles_and_ignoring_empty_nodes() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([
				((0, 0), GroundPosition(Vec3::new(-5., 0., -5.))),
				((1, 0), GroundPosition(Vec3::new(5., 0., -5.))),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 2),
				cell_count_z: new_valid!(CellCount, 2),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			Ok(SquareGridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), GroundPosition(Vec3::new(-7.5, 0., -7.5))),
					((0, 1), GroundPosition(Vec3::new(-7.5, 0., -2.5))),
					((1, 0), GroundPosition(Vec3::new(-2.5, 0., -7.5))),
					((1, 1), GroundPosition(Vec3::new(-2.5, 0., -2.5))),
					// old (1, 0)
					((2, 0), GroundPosition(Vec3::new(2.5, 0., -7.5))),
					((2, 1), GroundPosition(Vec3::new(2.5, 0., -2.5))),
					((3, 0), GroundPosition(Vec3::new(7.5, 0., -7.5))),
					((3, 1), GroundPosition(Vec3::new(7.5, 0., -2.5))),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: SquareGridContext {
					cell_count_x: new_valid!(CellCount, 4),
					cell_count_z: new_valid!(CellCount, 4),
					cell_distance: new_valid!(CellDistance, 5.),
				},
			}),
			resized
		);
	}

	#[test]
	fn subdivide_1() {
		let graph = SquareGridGraph {
			nodes: HashMap::from([
				((0, 0), GroundPosition(Vec3::new(-5., 0., -5.))),
				((0, 1), GroundPosition(Vec3::new(-5., 0., 5.))),
				((1, 0), GroundPosition(Vec3::new(5., 0., -5.))),
				((1, 1), GroundPosition(Vec3::new(5., 0., 5.))),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(0, 1), (1, 1)]),
			},
			context: SquareGridContext {
				cell_count_x: new_valid!(CellCount, 2),
				cell_count_z: new_valid!(CellCount, 2),
				cell_distance: new_valid!(CellDistance, 10.),
			},
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			Ok(SquareGridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), GroundPosition(Vec3::new(-7.5, 0., -7.5))),
					((0, 1), GroundPosition(Vec3::new(-7.5, 0., -2.5))),
					((1, 0), GroundPosition(Vec3::new(-2.5, 0., -7.5))),
					((1, 1), GroundPosition(Vec3::new(-2.5, 0., -2.5))),
					// old (1, 0)
					((2, 0), GroundPosition(Vec3::new(2.5, 0., -7.5))),
					((2, 1), GroundPosition(Vec3::new(2.5, 0., -2.5))),
					((3, 0), GroundPosition(Vec3::new(7.5, 0., -7.5))),
					((3, 1), GroundPosition(Vec3::new(7.5, 0., -2.5))),
					// old (0, 1)
					((0, 2), GroundPosition(Vec3::new(-7.5, 0., 2.5))),
					((0, 3), GroundPosition(Vec3::new(-7.5, 0., 7.5))),
					((1, 2), GroundPosition(Vec3::new(-2.5, 0., 2.5))),
					((1, 3), GroundPosition(Vec3::new(-2.5, 0., 7.5))),
					// old (1, 1)
					((2, 2), GroundPosition(Vec3::new(2.5, 0., 2.5))),
					((2, 3), GroundPosition(Vec3::new(2.5, 0., 7.5))),
					((3, 2), GroundPosition(Vec3::new(7.5, 0., 2.5))),
					((3, 3), GroundPosition(Vec3::new(7.5, 0., 7.5))),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([
						// old (0, 1)
						(0, 2),
						(0, 3),
						(1, 2),
						(1, 3),
						// old (1, 1)
						(2, 2),
						(2, 3),
						(3, 2),
						(3, 3),
					]),
				},
				context: SquareGridContext {
					cell_count_x: new_valid!(CellCount, 4),
					cell_count_z: new_valid!(CellCount, 4),
					cell_distance: new_valid!(CellDistance, 5.),
				},
			}),
			resized
		);
	}
}
