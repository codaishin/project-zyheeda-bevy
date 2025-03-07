use super::{
	GridGraph,
	grid_context::{GridContext, GridDefinition},
};
use crate::traits::{grid_start::GridStart, to_subdivided::ToSubdivided};
use bevy::utils::default;

impl ToSubdivided for GridGraph {
	fn to_subdivided(&self, subdivisions: u8) -> Self {
		match subdivisions {
			0 => self.clone(),
			_ => subdivide(self, subdivisions),
		}
	}
}

fn subdivide(source: &GridGraph, subdivisions: u8) -> GridGraph {
	let factor = subdivisions + 1;
	let GridContext(source_grid) = source.context;
	let mut subdivided: GridGraph = GridGraph {
		context: GridContext(GridDefinition {
			cell_count_x: source_grid.cell_count_x * factor as usize,
			cell_count_z: source_grid.cell_count_z * factor as usize,
			cell_distance: source_grid.cell_distance / factor as f32,
		}),
		..default()
	};
	let min = subdivided.context.grid_min();
	let mut translation = min;

	for x in 0..subdivided.context.0.cell_count_x {
		for z in 0..subdivided.context.0.cell_count_z {
			let source_key = source_key(x, z, factor);
			if source.nodes.contains_key(&source_key) {
				subdivided.nodes.insert((x as i32, z as i32), translation);
			}
			if source.extra.obstacles.contains(&source_key) {
				subdivided.extra.obstacles.insert((x as i32, z as i32));
			}
			translation.z += subdivided.context.0.cell_distance;
		}

		translation.x += subdivided.context.0.cell_distance;
		translation.z = min.z;
	}

	subdivided
}

fn source_key(x: usize, z: usize, factor: u8) -> (i32, i32) {
	let factor = factor as f32;

	((x as f32 / factor) as i32, (z as f32 / factor) as i32)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::grid_graph::{
		Obstacles,
		grid_context::{GridContext, GridDefinition, GridDefinitionError},
	};
	use bevy::math::Vec3;
	use std::collections::{HashMap, HashSet};

	#[test]
	fn subdivide_0_returns_same_graph() -> Result<(), GridDefinitionError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-5., 0., -5.)),
				((0, 1), Vec3::new(-5., 0., 5.)),
				((1, 0), Vec3::new(5., 0., -5.)),
				((1, 1), Vec3::new(5., 0., 5.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(0, 1), (1, 1)]),
			},
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 10.,
			})?,
		};

		let resized = graph.clone().to_subdivided(0);

		assert_eq!(graph, resized);
		Ok(())
	}

	#[test]
	fn subdivide_1_with_all_empty_nodes() -> Result<(), GridDefinitionError> {
		let graph = GridGraph {
			nodes: HashMap::from([]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 10.,
			})?,
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			GridGraph {
				nodes: HashMap::from([]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 4,
					cell_count_z: 4,
					cell_distance: 5.,
				})?,
			},
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_1_without_obstacles() -> Result<(), GridDefinitionError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-5., 0., -5.)),
				((0, 1), Vec3::new(-5., 0., 5.)),
				((1, 0), Vec3::new(5., 0., -5.)),
				((1, 1), Vec3::new(5., 0., 5.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 10.,
			})?,
		};

		let resized = graph.to_subdivided(1);

		assert_eq!(
			GridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), Vec3::new(-7.5, 0., -7.5)),
					((0, 1), Vec3::new(-7.5, 0., -2.5)),
					((1, 0), Vec3::new(-2.5, 0., -7.5)),
					((1, 1), Vec3::new(-2.5, 0., -2.5)),
					// old (1, 0)
					((2, 0), Vec3::new(2.5, 0., -7.5)),
					((2, 1), Vec3::new(2.5, 0., -2.5)),
					((3, 0), Vec3::new(7.5, 0., -7.5)),
					((3, 1), Vec3::new(7.5, 0., -2.5)),
					// old (0, 1)
					((0, 2), Vec3::new(-7.5, 0., 2.5)),
					((0, 3), Vec3::new(-7.5, 0., 7.5)),
					((1, 2), Vec3::new(-2.5, 0., 2.5)),
					((1, 3), Vec3::new(-2.5, 0., 7.5)),
					// old (1, 1)
					((2, 2), Vec3::new(2.5, 0., 2.5)),
					((2, 3), Vec3::new(2.5, 0., 7.5)),
					((3, 2), Vec3::new(7.5, 0., 2.5)),
					((3, 3), Vec3::new(7.5, 0., 7.5)),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 4,
					cell_count_z: 4,
					cell_distance: 5.,
				})?,
			},
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_1_without_obstacles_and_ignoring_empty_nodes() -> Result<(), GridDefinitionError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-5., 0., -5.)),
				((1, 0), Vec3::new(5., 0., -5.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([]),
			},
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 10.,
			})?,
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			GridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), Vec3::new(-7.5, 0., -7.5)),
					((0, 1), Vec3::new(-7.5, 0., -2.5)),
					((1, 0), Vec3::new(-2.5, 0., -7.5)),
					((1, 1), Vec3::new(-2.5, 0., -2.5)),
					// old (1, 0)
					((2, 0), Vec3::new(2.5, 0., -7.5)),
					((2, 1), Vec3::new(2.5, 0., -2.5)),
					((3, 0), Vec3::new(7.5, 0., -7.5)),
					((3, 1), Vec3::new(7.5, 0., -2.5)),
				]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 4,
					cell_count_z: 4,
					cell_distance: 5.,
				})?,
			},
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_1() -> Result<(), GridDefinitionError> {
		let graph = GridGraph {
			nodes: HashMap::from([
				((0, 0), Vec3::new(-5., 0., -5.)),
				((0, 1), Vec3::new(-5., 0., 5.)),
				((1, 0), Vec3::new(5., 0., -5.)),
				((1, 1), Vec3::new(5., 0., 5.)),
			]),
			extra: Obstacles {
				obstacles: HashSet::from([(0, 1), (1, 1)]),
			},
			context: GridContext::try_from(GridDefinition {
				cell_count_x: 2,
				cell_count_z: 2,
				cell_distance: 10.,
			})?,
		};

		let resized = graph.clone().to_subdivided(1);

		assert_eq!(
			GridGraph {
				nodes: HashMap::from([
					// old (0, 0)
					((0, 0), Vec3::new(-7.5, 0., -7.5)),
					((0, 1), Vec3::new(-7.5, 0., -2.5)),
					((1, 0), Vec3::new(-2.5, 0., -7.5)),
					((1, 1), Vec3::new(-2.5, 0., -2.5)),
					// old (1, 0)
					((2, 0), Vec3::new(2.5, 0., -7.5)),
					((2, 1), Vec3::new(2.5, 0., -2.5)),
					((3, 0), Vec3::new(7.5, 0., -7.5)),
					((3, 1), Vec3::new(7.5, 0., -2.5)),
					// old (0, 1)
					((0, 2), Vec3::new(-7.5, 0., 2.5)),
					((0, 3), Vec3::new(-7.5, 0., 7.5)),
					((1, 2), Vec3::new(-2.5, 0., 2.5)),
					((1, 3), Vec3::new(-2.5, 0., 7.5)),
					// old (1, 1)
					((2, 2), Vec3::new(2.5, 0., 2.5)),
					((2, 3), Vec3::new(2.5, 0., 7.5)),
					((3, 2), Vec3::new(7.5, 0., 2.5)),
					((3, 3), Vec3::new(7.5, 0., 7.5)),
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
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 4,
					cell_count_z: 4,
					cell_distance: 5.,
				})?,
			},
			resized
		);
		Ok(())
	}
}
