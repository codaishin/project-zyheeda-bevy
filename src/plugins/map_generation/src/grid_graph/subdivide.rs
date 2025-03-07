use super::{
	GridGraph,
	grid_context::{GridContext, GridDefinition},
};
use crate::traits::grid_start::GridStart;
use bevy::utils::default;
use common::errors::{Error, Level};

impl GridGraph {
	pub(crate) fn try_subdivide(self, factor: u8) -> Result<Self, SubdivisionByZero> {
		if factor == 0 {
			return Err(SubdivisionByZero);
		}

		if factor == 1 {
			return Ok(self);
		}

		Ok(self.subdivide(factor))
	}

	fn subdivide(self, factor: u8) -> GridGraph {
		let GridContext(old_grid) = self.context;
		let mut graph: GridGraph = GridGraph {
			context: GridContext(GridDefinition {
				cell_count_x: old_grid.cell_count_x * factor as usize,
				cell_count_z: old_grid.cell_count_z * factor as usize,
				cell_distance: old_grid.cell_distance / factor as f32,
			}),
			..default()
		};
		let min = graph.context.grid_min();
		let mut translation = min;

		for x in 0..graph.context.0.cell_count_x {
			for z in 0..graph.context.0.cell_count_z {
				let old = Self::old_key(x, z, factor);
				if self.nodes.contains_key(&old) {
					graph.nodes.insert((x as i32, z as i32), translation);
				}
				if self.extra.obstacles.contains(&old) {
					graph.extra.obstacles.insert((x as i32, z as i32));
				}
				translation.z += graph.context.0.cell_distance;
			}

			translation.x += graph.context.0.cell_distance;
			translation.z = min.z;
		}

		graph
	}

	fn old_key(x: usize, z: usize, factor: u8) -> (i32, i32) {
		let factor = factor as f32;

		((x as f32 / factor) as i32, (z as f32 / factor) as i32)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct SubdivisionByZero;

impl From<SubdivisionByZero> for Error {
	fn from(_: SubdivisionByZero) -> Self {
		Error {
			msg: "Tried to subdivide a grid with factor 0, which is not possible".to_owned(),
			lvl: Level::Error,
		}
	}
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
	fn subdivide_by_0_is_error() {
		let graph: GridGraph = GridGraph::default();

		let resized = graph.try_subdivide(0);

		assert_eq!(Err(SubdivisionByZero), resized);
	}

	#[test]
	fn subdivide_by_1_returns_same_graph() -> Result<(), GridDefinitionError> {
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

		let resized = graph.clone().try_subdivide(1);

		assert_eq!(Ok(graph), resized);
		Ok(())
	}

	#[test]
	fn subdivide_by_2_with_all_empty_nodes() -> Result<(), GridDefinitionError> {
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

		let resized = graph.clone().try_subdivide(2);

		assert_eq!(
			Ok(GridGraph {
				nodes: HashMap::from([]),
				extra: Obstacles {
					obstacles: HashSet::from([]),
				},
				context: GridContext::try_from(GridDefinition {
					cell_count_x: 4,
					cell_count_z: 4,
					cell_distance: 5.,
				})?,
			}),
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_by_2_without_obstacles() -> Result<(), GridDefinitionError> {
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

		let resized = graph.clone().try_subdivide(2);

		assert_eq!(
			Ok(GridGraph {
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
			}),
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_by_2_without_obstacles_and_ignoring_empty_nodes() -> Result<(), GridDefinitionError>
	{
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

		let resized = graph.clone().try_subdivide(2);

		assert_eq!(
			Ok(GridGraph {
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
			}),
			resized
		);
		Ok(())
	}

	#[test]
	fn subdivide_by_2() -> Result<(), GridDefinitionError> {
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

		let resized = graph.clone().try_subdivide(2);

		assert_eq!(
			Ok(GridGraph {
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
			}),
			resized
		);
		Ok(())
	}
}
